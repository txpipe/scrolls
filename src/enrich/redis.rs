use std::time::Duration;

use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};
use pallas::{
    codec::minicbor,
    ledger::traverse::{Era, MultiEraBlock, MultiEraTx},
};
use serde::Deserialize;
use redis::{Commands, Connection, RedisError};

use crate::{
    bootstrap, crosscut,
    model::{self, BlockContext},
    prelude::AppliesPolicy,
};

type InputPort = gasket::messaging::TwoPhaseInputPort<model::RawBlockPayload>;
type OutputPort = gasket::messaging::OutputPort<model::EnrichedBlockPayload>;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub redis_url: String,
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
            conn: None,
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
            Some("enrich-redis"),
        ));
    }
}

pub struct Worker {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    conn: Option<Connection>,
    input: InputPort,
    output: OutputPort,
    inserts_counter: gasket::metrics::Counter,
    remove_counter: gasket::metrics::Counter,
    matches_counter: gasket::metrics::Counter,
    mismatches_counter: gasket::metrics::Counter,
    blocks_counter: gasket::metrics::Counter,
}

impl Worker {
    #[inline]
    fn insert_produced_utxos(
        &mut self,
        txs: &[MultiEraTx],
    ) -> Result<(), crate::Error> {
        for tx in txs.iter() {
            for (idx, output) in tx.produces() {
                let key: String = format!("utxo.{}#{}", tx.hash(), idx);

                let era: u16 = tx.era().into();
                let body: Vec<u8> = output.encode();
                let value: Vec<u8> = minicbor::to_vec(&(era, body)).map_err(crate::Error::cbor)?;

                self.conn
                    .as_mut()
                    .unwrap()
                    .set::<String, Vec<u8>, ()>(key, value).map_err(crate::Error::storage)?;
            }
        }
        self.inserts_counter.inc(txs.len() as u64);

        Ok(())
    }
    
    #[inline]
    fn par_fetch_referenced_utxos(
        &mut self,
        txs: &[MultiEraTx],
    ) -> Result<BlockContext, crate::Error> {
        let mut ctx = BlockContext::default();

        let required: Vec<_> = txs
            .iter()
            .flat_map(|tx| tx.requires())
            .map(|input| input.output_ref())
            .collect();

        let required_keys: Vec<String> = required.iter().map(|utxo_ref| format!("utxo.{}", utxo_ref.to_string())).collect();

        let values: Result<Vec<Option<Vec<u8>>>, RedisError> = self.conn.as_mut().unwrap().mget(required_keys);

        match values {
            Ok(values) => {
                for (utxo_ref, ivec_opt) in required.into_iter().zip(values.into_iter()) {
                    if let Some(ivec) = ivec_opt {
                        let (era, cbor): (u16, Vec<u8>) = minicbor::decode(&ivec).map_err(crate::Error::cbor)?;
                        let era: Era = era.try_into().map_err(crate::Error::storage)?;
                        ctx.import_ref_output(&utxo_ref, era, cbor);
                        self.matches_counter.inc(1);
                    } else {
                        self.mismatches_counter.inc(1);
                    }
                }
                Ok(ctx)
            }
            Err(_) => Ok(ctx),
        }
    }
    
    fn remove_consumed_utxos(&mut self, txs: &[MultiEraTx]) -> Result<(), crate::Error> {
        let keys: Vec<_> = txs
            .iter()
            .flat_map(|tx| tx.consumes())
            .map(|i| format!("utxo.{}", i.output_ref()))
            .collect();
    
        for key in keys.iter() {
            self.conn.as_mut().unwrap().del::<String, ()>(key.to_string()).map_err(crate::Error::storage)?;
        }
    
        self.remove_counter.inc(keys.len() as u64);
    
        Ok(())
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
    
                let txs = block.txs();
    
                // first we insert new utxo
                // produced in this block
                self.insert_produced_utxos(&txs).or_restart()?;

                // then we fetch referenced utxo in this block
                let ctx = self.par_fetch_referenced_utxos(&txs).or_restart()?;

                // and finally we remove utxos consumed by the block
                self.remove_consumed_utxos(&txs).or_restart()?;

                self.output
                    .send(model::EnrichedBlockPayload::roll_forward(cbor, ctx))?;

                self.blocks_counter.inc(1);
            }
            model::RawBlockPayload::RollBack(x) => {
                self.output
                    .send(model::EnrichedBlockPayload::roll_back(x))?;
            }
        };

        self.input.commit();
        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        self.conn = redis::Client::open(self.config.redis_url.clone())
            .and_then(|c| c.get_connection())
            .or_retry()?
            .into();

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        // Redis connections are closed automatically when they go out of scope,
        // so there is no need for explicit teardown logic.

        Ok(())
    }
}