use std::time::Duration;

use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};

use log::{log, warn};

use pallas::{
    codec::minicbor,
    ledger::traverse::{Era, MultiEraBlock, MultiEraTx, OutputRef},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use sled::IVec;

use crate::{
    bootstrap, crosscut,
    model::{self, BlockContext},
    prelude::AppliesPolicy,
};

type InputPort = gasket::messaging::TwoPhaseInputPort<model::RawBlockPayload>;
type OutputPort = gasket::messaging::OutputPort<model::EnrichedBlockPayload>;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub db_path: String,
    pub rollback_db_path: Option<String>,
}

impl Config {
    pub fn boostrapper(mut self, policy: &crosscut::policies::RuntimePolicy, blocks: &crosscut::historic::BlockConfig) -> Bootstrapper {
        self.rollback_db_path = Some(blocks.rollback_db_path.clone());

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
            enrich_db: None,
            rollback_db: None,
            input: self.input,
            output: self.output,
            inserts_counter: Default::default(),
            remove_counter: Default::default(),
            matches_counter: Default::default(),
            mismatches_counter: Default::default(),
            blocks_counter: Default::default(),
        };

        pipeline.register_stage(spawn_stage(
            worker,
            gasket::runtime::Policy {
                tick_timeout: Some(Duration::from_secs(600)),
                ..Default::default()
            },
            Some("enrich-sled"),
        ));
    }
}

pub struct Worker {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    enrich_db: Option<sled::Db>,
    rollback_db: Option<sled::Db>,
    input: InputPort,
    output: OutputPort,
    inserts_counter: gasket::metrics::Counter,
    remove_counter: gasket::metrics::Counter,
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
fn fetch_referenced_utxo<'a>(
    db: &sled::Db,
    utxo_ref: &OutputRef,
) -> Result<Option<(OutputRef, Era, Vec<u8>)>, crate::Error> {
    if let Some(ivec) = db
        .get(utxo_ref.to_string().as_bytes())
        .map_err(crate::Error::storage)?
    {
        let SledTxValue(era, cbor) = ivec.try_into().map_err(crate::Error::storage)?;
        let era: Era = era.try_into().map_err(crate::Error::storage)?;
        Ok(Some((utxo_ref.clone(), era, cbor)))
    } else {
        Ok(None)
    }
}

impl Worker {
    fn db_refs_all(&self) -> Result<Option<(&sled::Db, &sled::Db)>, ()> {
        match (self.db_ref_enrich(), self.db_ref_rollback()) {
            (Some(db), Some(consumed_ring)) => {
                Ok(Some((db, consumed_ring)))
            },
            _ => Err(())
        }
    }

    fn db_ref_enrich(&self) -> Option<&sled::Db> {
        match self.enrich_db.as_ref() {
            None => None,
            Some(db) => Some(db)
        }
    }

    fn db_ref_rollback(&self) -> Option<&sled::Db> {
        match self.rollback_db.as_ref() {
            None => None,
            Some(db) => Some(db)
        }
    }

    fn insert_produced_utxos(&self, db: &sled::Db, txs: &[MultiEraTx]) -> Result<(), crate::Error> {
        let mut insert_batch = sled::Batch::default();

        for tx in txs.iter() {
            for (idx, output) in tx.produces() {
                let key = format!("{}#{}", tx.hash(), idx);

                let era = tx.era().into();
                let body = output.encode();
                let value: IVec = SledTxValue(era, body).try_into()?;

                insert_batch.insert(key.as_bytes(), value)
            }
        }

        let batch_results = db.apply_batch(insert_batch).or_retry();

        self.inserts_counter.inc(txs.len() as u64);

        batch_results.map_err(crate::Error::storage)
    }

    fn remove_produced_utxos(&self, db: &sled::Db, txs: &[MultiEraTx]) -> Result<(), crate::Error> {
        let mut remove_batch = sled::Batch::default();

        for tx in txs.iter() {
            for (idx, _) in tx.produces() {
                remove_batch.remove(format!("{}#{}", tx.hash(), idx).as_bytes());
            }
        }

        self.remove_counter.inc(txs.len() as u64);
        db.apply_batch(remove_batch).map_err(crate::Error::storage)
    }

    #[inline]
    fn par_fetch_referenced_utxos(
        &self,
        db: &sled::Db,
        txs: &[MultiEraTx],
    ) -> Result<BlockContext, crate::Error> {
        let mut ctx = BlockContext::default();

        let required: Vec<_> = txs
            .iter()
            .flat_map(|tx| tx.requires())
            .map(|input| input.output_ref())
            .collect();

        let matches: Result<Vec<_>, crate::Error> = required
            .par_iter()
            .map(|utxo_ref| fetch_referenced_utxo(db, utxo_ref))
            .collect();

        for m in matches? {
            if let Some((key, era, cbor)) = m {
                ctx.import_ref_output(&key, era, cbor);
                self.matches_counter.inc(1);
            } else {
                self.mismatches_counter.inc(1);
            }
        }

        Ok(ctx)
    }

    fn get_removed(&self, consumed_ring: &sled::Db, key: &[u8]) -> Result<Option<IVec>, crate::Error> {
        consumed_ring
            .get(key)
            .map_err(crate::Error::storage)
    }

    fn remove_consumed_utxos(&self, db: &sled::Db, consumed_ring: &sled::Db, txs: &[MultiEraTx]) -> Result<(), ()> {
        let mut remove_batch = sled::Batch::default();
        let mut current_values_batch = sled::Batch::default();

        let keys: Vec<_> = txs
            .iter()
            .flat_map(|tx| tx.consumes())
            .map(|i| i.output_ref())
            .collect();

        for key in keys.iter() {
            if let Some(current_value) = db
                .get(key.to_string())
                .map_err(crate::Error::storage).unwrap() {
                current_values_batch.insert(key.to_string().as_bytes(), current_value);
            }

            remove_batch.remove(key.to_string().as_bytes());
        }

        let result: Result<(), ()> = match (db.apply_batch(remove_batch),
               consumed_ring.apply_batch(current_values_batch)) {
            (Ok(()), Ok(())) => Ok(()),
            _ => Err(())
        };

        self.remove_counter.inc(keys.len() as u64);

        result
    }

    fn replace_consumed_utxos(&self, db: &sled::Db, consumed_ring: &sled::Db, txs: &[MultiEraTx]) -> Result<(), crate::Error> {
        let mut insert_batch = sled::Batch::default();
        let mut remove_batch = sled::Batch::default();

        let keys: Vec<_> = txs
            .iter()
            .flat_map(|tx| tx.consumes())
            .map(|i| i.output_ref())
            .collect();

        for key in keys.iter().rev() {
            if let Ok(Some(existing_value)) = self.get_removed(consumed_ring, key.to_string().as_bytes()) {
                insert_batch.insert(key.to_string().as_bytes(), existing_value);
                remove_batch.remove(key.to_string().as_bytes());
            }

        }

        let result = match (db.apply_batch(insert_batch), consumed_ring.apply_batch(remove_batch)) {
            (Ok(_), Ok(_)) => Ok(()),
            _ => Err(crate::Error::storage("failed to roll back consumed utxos"))
        };

        self.inserts_counter.inc(txs.len() as u64);

        result
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("enrich_inserts", &self.inserts_counter)
            .with_counter("enrich_removes", &self.remove_counter)
            .with_counter("enrich_matches", &self.matches_counter)
            .with_counter("enrich_mismatches", &self.mismatches_counter)
            .with_counter("enrich_blocks", &self.blocks_counter)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv_or_idle()?;
        let all_dbs = self.db_refs_all();
        if let Err(_) = all_dbs {
            return Err(gasket::error::Error::WorkPanic("database error".into()))
        }

        let all_dbs = all_dbs.unwrap();

        if let Some((db, consumed_ring)) = all_dbs {
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

                    let txs = &block.txs();

                    self.insert_produced_utxos(db, txs).or_panic()?;
                    let ctx = self.par_fetch_referenced_utxos(db, &txs).or_panic()?;

                    // and finally we remove utxos consumed by the block
                    self.remove_consumed_utxos(db, consumed_ring, &txs).expect("todo panic");

                    self.output
                        .send(model::EnrichedBlockPayload::roll_forward(cbor, ctx))?;

                }
                model::RawBlockPayload::RollBack(cbor) => {
                    let block = MultiEraBlock::decode(&cbor)
                        .map_err(crate::Error::cbor)
                        .apply_policy(&self.policy);

                    match block {
                        Ok(block) => {
                            let block = match block {
                                Some(x) => x,
                                None => return Ok(gasket::runtime::WorkOutcome::Partial),
                            };

                            let txs = block.txs();

                            // Revert Anything to do with this block
                            self.remove_produced_utxos(db, &txs).expect("todo: panic error");
                            self.replace_consumed_utxos(db, consumed_ring, &txs).expect("todo: panic error");

                            let ctx = self.par_fetch_referenced_utxos(db, &txs)
                                .or_restart()?;

                            self.output
                                .send(model::EnrichedBlockPayload::roll_back(cbor, ctx))?;
                        }
                        Err(e) => {
                            return Ok(gasket::runtime::WorkOutcome::Partial)
                        }
                    }

                    self.blocks_counter.inc(1);
                }
            };
        }

        self.input.commit();
        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let enrich_db = sled::open(&self.config.db_path).or_retry()?;
        let rollback_db = sled::open(self.config.rollback_db_path.clone().unwrap_or_default()).or_retry()?;

        self.enrich_db = Some(enrich_db);
        self.rollback_db = Some(rollback_db);

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        Ok(())
    }
}
