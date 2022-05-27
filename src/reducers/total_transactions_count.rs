use gasket::runtime::{spawn_stage, WorkOutcome};
use pallas::ledger::primitives::byron;
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};


type InputPort = gasket::messaging::InputPort<model::ChainSyncCommandEx>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Worker {
    config: Config,
    input: InputPort,
    output: OutputPort,
    ops_count: gasket::metrics::Counter,
}

impl Worker {
    fn increment_key(
        &mut self
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => prefix.to_string(),
            None => "total_transactions_count".to_string()
        };

        let crdt = model::CRDTCommand::PNCounter(key, 1.to_string());

        self.output.send(gasket::messaging::Message::from(crdt))?;
        self.ops_count.inc(1);

        Ok(())
    }

    fn reduce_block(&mut self, block: &model::MultiEraBlock) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(byron::Block::MainBlock(x)) => x
                .body
                .tx_payload
                .iter()
                .map(|_tx| self.increment_key())
                .collect(),

                model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(block) => block
                .1
                .transaction_bodies
                .iter()
                .map(|_tx| self.increment_key())
                .collect(),
        }
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv()?;

        match msg.payload {
            model::ChainSyncCommandEx::RollForward(block) => self.reduce_block(&block)?,
            model::ChainSyncCommandEx::RollBack(point) => {
                log::warn!("rollback requested for {:?}", point);
            }
        }

        Ok(WorkOutcome::Partial)
    }
}

impl super::Pluggable for Worker {
    fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    fn borrow_output_port(&mut self) -> &'_ mut OutputPort {
        &mut self.output
    }

    fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        pipeline.register_stage("total_transactions_count", spawn_stage(self, Default::default()));
    }
}

impl super::IntoPlugin for Config {
    fn plugin(
        self,
        _chain: &crosscut::ChainWellKnownInfo,
        _intersect: &crosscut::IntersectConfig,
    ) -> super::Plugin {
        let worker = Worker {
            config: self,
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
        };

        super::Plugin::TotalTransactionsCount(worker)
    }
}
