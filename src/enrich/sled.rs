use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};

use pallas::{codec::minicbor, ledger::traverse::MultiEraBlock};
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

        pipeline.register_stage("enrich-sled", spawn_stage(worker, Default::default()));
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

impl Worker {
    fn track_block_txs(&self, block: &MultiEraBlock) -> Result<BlockContext, crate::Error> {
        let db = self.db.as_ref().unwrap();
        let mut ctx = BlockContext::default();

        for tx in &block.txs() {
            let hash = tx.hash();

            let era = tx.era().into();
            let body = tx.encode().map_err(crate::Error::cbor)?;
            let value: IVec = SledTxValue(era, body).try_into()?;
            db.insert(hash, value).map_err(crate::Error::storage)?;
            self.inserts_counter.inc(1);

            for input in tx.inputs() {
                if let Some(tx_ref) = input.output_ref() {
                    let tx_id = tx_ref.tx_id();

                    if let Some(ivec) = db.get(tx_id).map_err(crate::Error::storage)? {
                        let SledTxValue(era, cbor) =
                            ivec.try_into().map_err(crate::Error::storage)?;
                        let era = era.try_into().map_err(crate::Error::storage)?;
                        ctx.import_ref_tx(tx_id, era, cbor);
                        self.matches_counter.inc(1);
                    } else {
                        self.mismatches_counter.inc(1);
                    }
                }
            }
        }

        Ok(ctx)
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
