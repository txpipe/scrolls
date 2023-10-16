use pallas::{
    codec::minicbor,
    ledger::traverse::{Era, MultiEraBlock, MultiEraTx, OutputRef},
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use sled::{Db, IVec};

use gasket::framework::*;

use crate::framework::{
    model::{BlockContext, EnrichedBlockPayload, RawBlockPayload},
    Context, EnrichInputPort, EnrichOutputPort, Error,
};

pub struct Worker {
    db: Db,
}
impl Worker {
    #[inline]
    fn insert_produced_utxos(&self, db: &sled::Db, txs: &[MultiEraTx]) -> Result<(), Error> {
        let mut insert_batch = sled::Batch::default();

        for tx in txs.iter() {
            for (idx, output) in tx.produces() {
                let key: IVec = format!("{}#{}", tx.hash(), idx).as_bytes().into();

                let era = tx.era().into();
                let body = output.encode();
                let value: IVec = SledTxValue(era, body).try_into()?;

                insert_batch.insert(key, value)
            }
        }

        db.apply_batch(insert_batch).map_err(Error::storage)?;

        // self.inserts_counter.inc(txs.len() as u64);

        Ok(())
    }

    #[inline]
    fn par_fetch_referenced_utxos(
        &self,
        db: &sled::Db,
        txs: &[MultiEraTx],
    ) -> Result<BlockContext, Error> {
        let mut ctx = BlockContext::default();

        let required: Vec<_> = txs
            .iter()
            .flat_map(|tx| tx.requires())
            .map(|input| input.output_ref())
            .collect();

        let matches: Result<Vec<_>, Error> = required
            .par_iter()
            .map(|utxo_ref| fetch_referenced_utxo(db, utxo_ref))
            .collect();

        for m in matches? {
            if let Some((key, era, cbor)) = m {
                ctx.import_ref_output(&key, era, cbor);
                // self.matches_counter.inc(1);
            }

            // else {
            //     self.mismatches_counter.inc(1);
            // }
        }

        Ok(ctx)
    }

    fn remove_consumed_utxos(&self, db: &sled::Db, txs: &[MultiEraTx]) -> Result<(), Error> {
        let keys: Vec<_> = txs
            .iter()
            .flat_map(|tx| tx.consumes())
            .map(|i| i.output_ref())
            .collect();

        for key in keys.iter() {
            db.remove(key.to_string().as_bytes())
                .map_err(Error::storage)?;
        }

        // self.remove_counter.inc(keys.len() as u64);

        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let db = sled::open(&stage.config.db_path).or_panic()?;
        Ok(Self { db })
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<RawBlockPayload>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(
        &mut self,
        unit: &RawBlockPayload,
        stage: &mut Stage,
    ) -> Result<(), WorkerError> {
        match unit {
            RawBlockPayload::RollForward(cbor) => {
                let block = MultiEraBlock::decode(&cbor)
                    .map_err(Error::cbor)
                    // .apply_policy(&self.policy)
                    .or_panic()?;

                let txs = block.txs();

                // first we insert new utxo produced in this block
                self.insert_produced_utxos(&self.db, &txs).or_restart()?;

                // then we fetch referenced utxo in this block
                let ctx = self
                    .par_fetch_referenced_utxos(&self.db, &txs)
                    .or_restart()?;

                // and finally we remove utxos consumed by the block
                self.remove_consumed_utxos(&self.db, &txs).or_restart()?;

                let evt = EnrichedBlockPayload::roll_forward(cbor.clone(), ctx);

                stage.output.send(evt).await.or_retry()?;
            }
            RawBlockPayload::RollBack(point) => stage
                .output
                .send(EnrichedBlockPayload::roll_back(point.clone()))
                .await
                .or_retry()?,
        }

        stage.ops_count.inc(1);

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "enrich-sled", unit = "RawBlockPayload", worker = "Worker")]
pub struct Stage {
    config: Config,
    pub input: EnrichInputPort,
    pub output: EnrichOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

#[derive(Default, Deserialize)]
pub struct Config {
    pub db_path: String,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
        };

        Ok(stage)
    }
}

struct SledTxValue(u16, Vec<u8>);

impl TryInto<IVec> for SledTxValue {
    type Error = Error;

    fn try_into(self) -> Result<IVec, Self::Error> {
        let SledTxValue(era, body) = self;

        minicbor::to_vec((era, body))
            .map(|x| IVec::from(x))
            .map_err(Error::cbor)
    }
}

impl TryFrom<IVec> for SledTxValue {
    type Error = Error;

    fn try_from(value: IVec) -> Result<Self, Self::Error> {
        let (tag, body): (u16, Vec<u8>) = minicbor::decode(&value).map_err(Error::cbor)?;

        Ok(SledTxValue(tag, body))
    }
}

#[inline]
fn fetch_referenced_utxo<'a>(
    db: &sled::Db,
    utxo_ref: &OutputRef,
) -> Result<Option<(OutputRef, Era, Vec<u8>)>, Error> {
    if let Some(ivec) = db.get(utxo_ref.to_string()).map_err(Error::storage)? {
        let SledTxValue(era, cbor) = ivec.try_into().map_err(Error::storage)?;
        let era: Era = era.try_into().map_err(Error::storage)?;
        return Ok(Some((utxo_ref.clone(), era, cbor)));
    }

    Ok(None)
}
