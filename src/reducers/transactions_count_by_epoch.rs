use gasket::runtime::{spawn_stage, WorkOutcome};
use serde::Deserialize;

use pallas::ledger::primitives::byron;
use crate::{bootstrap, crosscut::{self, EpochCalculator}, model};

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
    shelley_known_slot: u64,
    shelley_epoch_length: u64,
    byron_epoch_length: u64,
    byron_slot_length: u64, 
    ops_count: gasket::metrics::Counter,
}

impl Worker {

    fn reduce_alonzo_compatible_tx(
        &mut self,
        slot: u64
    ) -> Result<(), gasket::error::Error> {

        let epoch_no = EpochCalculator::get_shelley_epoch_no_for_absolute_slot(
            self.shelley_known_slot,
            self.shelley_epoch_length,
            slot
        );

        return self.increment_key(epoch_no);
    }

    fn reduce_byron_compatible_tx(
        &mut self,
        slot: u64
    ) -> Result<(), gasket::error::Error> {

        let epoch_no = EpochCalculator::get_byron_epoch_no_for_absolute_slot(
            self.byron_epoch_length,
            self.byron_slot_length,
            slot
        );

        return self.increment_key(epoch_no);
    }

    fn increment_key(
        &mut self,
        epoch_no: u64
    ) -> Result<(), gasket::error::Error> {

        let prefix = match &self.config.key_prefix {
            Some(prefix) => prefix,
            None => "transactions_by_epoch",
        };

        let key = format!("{}.{}", prefix, epoch_no.to_string());

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
                .map(|_tx| self.reduce_byron_compatible_tx(x.header.consensus_data.0.to_abs_slot()))
                .collect(),

                model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(x) => x
                .1
                .transaction_bodies
                .iter()
                .map(|_tx| self.reduce_alonzo_compatible_tx(x.1.header.header_body.slot))
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
        pipeline.register_stage("transactions_count_by_epoch", spawn_stage(self, Default::default()));
    }
}

impl super::IntoPlugin for Config {
    fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        _intersect: &crosscut::IntersectConfig,
    ) -> super::Plugin {

        let worker = Worker {
            config: self,
            shelley_known_slot: chain.shelley_known_slot.clone() as u64,
            shelley_epoch_length: chain.shelley_epoch_length.clone() as u64,
            byron_epoch_length: chain.byron_epoch_length.clone() as u64,
            byron_slot_length: chain.byron_slot_length.clone() as u64,
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
        };

        super::Plugin::TransactionsCountByEpoch(worker)
    }
}
