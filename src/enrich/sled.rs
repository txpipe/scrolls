use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};

use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{bootstrap, model};

type InputPort = gasket::messaging::InputPort<model::ChainSyncCommandEx>;
type OutputPort = gasket::messaging::OutputPort<model::ChainSyncCommandEx>;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub db_path: String,
}

impl Config {
    pub fn boostrapper(self) -> Bootstrapper {
        Bootstrapper {
            config: self,
            input: Default::default(),
            output: Default::default(),
        }
    }
}

pub struct Bootstrapper {
    config: Config,
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
            config: self.config.clone(),
            db: None,
            input: self.input,
            output: self.output,
        };

        pipeline.register_stage("enrich-sled", spawn_stage(worker, Default::default()));
    }
}

pub struct Worker {
    config: Config,
    db: Option<sled::Db>,
    input: InputPort,
    output: OutputPort,
}

impl Worker {
    fn track_block_txs(&self, cbor: &[u8]) -> Result<(), crate::Error> {
        let block = MultiEraBlock::decode(cbor).map_err(crate::Error::cbor)?;

        for tx in block.tx_iter() {
            let hash = tx.hash();

            let cbor = tx.encode().map_err(crate::Error::cbor)?;

            self.db
                .unwrap()
                .insert(hash, cbor)
                .map_err(crate::Error::storage)?;
        }

        Ok(())
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new().build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv()?;

        match &msg.payload {
            model::ChainSyncCommandEx::RollForward(cbor) => {
                self.track_block_txs(cbor).or_work_err()?;
                self.output.send(msg)
            }
            model::ChainSyncCommandEx::RollBack(_) => self.output.send(msg),
        };

        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let db = sled::open(self.config.db_path).or_work_err()?;
        self.db = Some(db);

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        match self.db {
            Some(db) => {
                db.flush().or_work_err()?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}
