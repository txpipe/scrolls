use std::time::Duration;

use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};

use pallas::{
    codec::minicbor,
    ledger::traverse::{Era, MultiEraBlock, MultiEraTx},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use sled::IVec;

use crate::{
    bootstrap, crosscut,
    model::{self, BlockContext},
    prelude::AppliesPolicy,
};

type InputPort = gasket::messaging::InputPort<model::RawBlockPayload>;
type OutputPort = gasket::messaging::OutputPort<model::EnrichedBlockPayload>;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub db_path: String,
}

impl Config {
    pub fn boostrapper(self, policy: &crosscut::policies::RuntimePolicy) -> Bootstrapper {
        Bootstrapper {
            config: self,
            policy: policy.clone(),
            input: Default::default(),
            output: Default::default(),
        }
    }
}

pub struct Bootstrapper {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    input: InputPort,
    output: OutputPort,
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort {
        &mut self.output
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        let worker = Worker {
            config: self.config,
            policy: self.policy,
            db: None,
            input: self.input,
            output: self.output,
            inserts_counter: Default::default(),
            matches_counter: Default::default(),
            mismatches_counter: Default::default(),
            blocks_counter: Default::default(),
        };

        pipeline.register_stage(
            "enrich-sled",
            spawn_stage(
                worker,
                gasket::runtime::Policy {
                    tick_timeout: Some(Duration::from_secs(120)),
                    ..Default::default()
                },
            ),
        );
    }
}

pub struct Worker {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    db: Option<sled::Db>,
    input: InputPort,
    output: OutputPort,
    inserts_counter: gasket::metrics::Counter,
    matches_counter: gasket::metrics::Counter,
    mismatches_counter: gasket::metrics::Counter,
    blocks_counter: gasket::metrics::Counter,
}

struct SledTxValue(u16, Vec<u8>);

impl TryInto<IVec> for SledTxValue {
    type Error = crate::Error;

    fn try_into(self) -> Result<IVec, Self::Error> {
        let SledTxValue(era, body) = self;
        minicbor::to_vec((era, body))
            .map(|x| IVec::from(x))
            .map_err(crate::Error::cbor)
    }
}

impl TryFrom<IVec> for SledTxValue {
    type Error = crate::Error;

    fn try_from(value: IVec) -> Result<Self, Self::Error> {
        let (tag, body): (u16, Vec<u8>) = minicbor::decode(&value).map_err(crate::Error::cbor)?;

        Ok(SledTxValue(tag, body))
    }
}

#[inline]
fn insert_new_txs(db: &sled::Db, txs: &[MultiEraTx]) -> Result<(), crate::Error> {
    let mut insert_batch = sled::Batch::default();

    for tx in txs.iter() {
        let key: IVec = tx.hash().to_vec().into();

        let era = tx.era().into();
        let body = tx.encode().map_err(crate::Error::cbor)?;
        let value: IVec = SledTxValue(era, body).try_into()?;

        insert_batch.insert(key, value)
    }

    db.apply_batch(insert_batch)
        .map_err(crate::Error::storage)?;

    Ok(())
}

impl Worker {
    #[inline]
    fn fetch_referenced_txs(
        &self,
        db: &sled::Db,
        txs: &[MultiEraTx],
    ) -> Result<BlockContext, crate::Error> {
        let mut ctx = BlockContext::default();

        let inputs: Vec<_> = txs
            .iter()
            .flat_map(|tx| tx.inputs())
            .filter_map(|input| input.output_ref().map(|r| r.tx_id().clone()))
            .collect();

        let matches: Result<Vec<_>, crate::Error> = inputs
            .par_iter()
            .map(|tx_id| {
                if let Some(ivec) = db.get(tx_id).map_err(crate::Error::storage)? {
                    let SledTxValue(era, cbor) = ivec.try_into().map_err(crate::Error::storage)?;
                    let era: Era = era.try_into().map_err(crate::Error::storage)?;
                    Ok(Some((tx_id, era, cbor)))
                } else {
                    Ok(None)
                }
            })
            .collect();

        for m in matches? {
            if let Some((tx_id, era, cbor)) = m {
                ctx.import_ref_tx(tx_id, era, cbor);
                self.matches_counter.inc(1);
            } else {
                self.mismatches_counter.inc(1);
            }
        }

        Ok(ctx)
    }

    fn track_block_txs(&self, block: &MultiEraBlock) -> Result<BlockContext, crate::Error> {
        let db = self.db.as_ref().unwrap();

        let txs = block.txs();

        insert_new_txs(db, &txs)?;
        self.inserts_counter.inc(txs.len() as u64);

        self.fetch_referenced_txs(&db, &txs)
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("enrich_inserts", &self.inserts_counter)
            .with_counter("enrich_matches", &self.matches_counter)
            .with_counter("enrich_mismatches", &self.mismatches_counter)
            .with_counter("enrich_blocks", &self.blocks_counter)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv_or_idle()?;

        match msg.payload {
            model::RawBlockPayload::RollForward(cbor) => {
                let block = MultiEraBlock::decode(&cbor)
                    .map_err(crate::Error::cbor)
                    .apply_policy(&self.policy)
                    .or_panic()?;

                let block = match block {
                    Some(x) => x,
                    None => return Ok(gasket::runtime::WorkOutcome::Partial),
                };

                let ctx = self.track_block_txs(&block).or_restart()?;

                self.output
                    .send(model::EnrichedBlockPayload::roll_forward(cbor, ctx))?;

                self.blocks_counter.inc(1);
            }
            model::RawBlockPayload::RollBack(x) => {
                self.output
                    .send(model::EnrichedBlockPayload::roll_back(x))?;
            }
        };

        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let db = sled::open(&self.config.db_path).or_retry()?;
        self.db = Some(db);

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        match &self.db {
            Some(db) => {
                db.flush().or_panic()?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}
