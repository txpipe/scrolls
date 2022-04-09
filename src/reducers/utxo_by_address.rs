use gasket::{error::AsWorkError, runtime::WorkOutcome};
use pallas::ledger::primitives::{alonzo, byron};

use crate::{
    model::{self, MultiEraBlock},
    sources::n2n::ChainSyncCommandEx,
};

pub struct Config {
    pub hrp: String,
}

pub struct Worker {
    config: Config,
    pub input: gasket::messaging::InputPort<ChainSyncCommandEx>,
    pub output: gasket::messaging::OutputPort<model::CRDTCommand>,
}

impl Worker {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            input: Default::default(),
            output: Default::default(),
        }
    }

    fn reduce_byron_tx(&mut self, tx: &byron::TxPayload) -> Result<(), gasket::error::Error> {
        let hash = tx.transaction.to_hash();

        tx.transaction
            .outputs
            .iter()
            .enumerate()
            .map(move |(idx, tx)| {
                let key = tx.address.to_addr_string().or_work_err()?;
                let member = format!("{}:{}", hash, idx);
                let crdt = model::CRDTCommand::TwoPhaseSetAdd(key, member);

                self.output.send(gasket::messaging::Message::from(crdt))
            })
            .collect()
    }

    fn reduce_alonzo_compatible_tx(
        &mut self,
        tx: &alonzo::TransactionBody,
    ) -> Result<(), gasket::error::Error> {
        let hash = tx.to_hash();

        tx.iter()
            .filter_map(|b| match b {
                alonzo::TransactionBodyComponent::Outputs(o) => Some(o),
                _ => None,
            })
            .flat_map(|o| o.iter())
            .enumerate()
            .map(move |(idx, output)| {
                let key = output.to_bech32_address(&self.config.hrp).or_work_err()?;
                let member = format!("{}:{}", hash, idx);
                let crdt = model::CRDTCommand::TwoPhaseSetAdd(key, member);

                self.output.send(gasket::messaging::Message::from(crdt))
            })
            .collect()
    }

    fn reduce_block(&mut self, block: MultiEraBlock) -> Result<(), gasket::error::Error> {
        match block {
            MultiEraBlock::Byron(byron::Block::MainBlock(x)) => x
                .body
                .tx_payload
                .iter()
                .map(|tx| self.reduce_byron_tx(tx))
                .collect(),
            MultiEraBlock::Byron(_) => Ok(()),
            MultiEraBlock::AlonzoCompatible(x) => {
                x.1.transaction_bodies
                    .iter()
                    .map(|tx| self.reduce_alonzo_compatible_tx(tx))
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
            ChainSyncCommandEx::RollForward(block) => self.reduce_block(block)?,
            ChainSyncCommandEx::RollBack(point) => {
                log::warn!("rollback requested for {:?}", point);
            }
        }

        Ok(WorkOutcome::Partial)
    }
}
