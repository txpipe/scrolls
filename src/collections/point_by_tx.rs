use gasket::runtime::{spawn_stage, WorkOutcome};
use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::byron;
use serde::Deserialize;

use crate::{bootstrap, model};

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
}

impl Worker {
    fn send_set_add(
        &mut self,
        tx_hash: Hash<32>,
        block_slot: u64,
        block_hash: Hash<32>,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, tx_hash),
            None => format!("{}", tx_hash),
        };

        let member = format!("{},{}", block_slot, block_hash);
        let crdt = model::CRDTCommand::GrowOnlySetAdd(key, member);

        self.output.send(gasket::messaging::Message::from(crdt))
    }

    fn reduce_block(&mut self, block: &model::MultiEraBlock) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(byron::Block::MainBlock(x)) => {
                let hash = x.header.to_hash();
                let slot = x.header.consensus_data.0.to_abs_slot();

                x.body
                    .tx_payload
                    .iter()
                    .map(|tx| tx.transaction.to_hash())
                    .map(|tx| self.send_set_add(tx, slot, hash))
                    .collect()
            }
            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(x) => {
                let slot = x.1.header.header_body.slot;
                let hash = x.1.header.header_body.block_body_hash;

                x.1.transaction_bodies
                    .iter()
                    .map(|tx| tx.to_hash())
                    .map(|tx| self.send_set_add(tx, slot, hash))
                    .collect()
            }
        }
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new().build()
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
        pipeline.register_stage("point_by_tx", spawn_stage(self, Default::default()));
    }
}

impl From<Config> for Worker {
    fn from(other: Config) -> Self {
        Self {
            config: other,
            input: Default::default(),
            output: Default::default(),
        }
    }
}
