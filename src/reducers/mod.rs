use gasket::runtime::{spawn_stage, WorkOutcome};

use crate::{
    bootstrap,
    model::{self, CRDTCommand, MultiEraBlock},
};

type InputPort = gasket::messaging::InputPort<model::ChainSyncCommandEx>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

pub mod point_by_tx;
pub mod pool_by_stake;
pub mod total_transactions_count;
pub mod total_transactions_count_by_contract_addresses;
pub mod transactions_count_by_contract_address;
pub mod transactions_count_by_contract_address_by_epoch;
pub mod transactions_count_by_epoch;
pub mod utxo_by_address;

pub enum Plugin {
    UtxoByAddress(utxo_by_address::Reducer),
    PointByTx(point_by_tx::Reducer),
    PoolByStake(pool_by_stake::Reducer),
    TotalTransactionsCount(total_transactions_count::Reducer),
    TransactionsCountByEpoch(transactions_count_by_epoch::Reducer),
    TransactionsCountByContractAddress(transactions_count_by_contract_address::Reducer),
    TransactionsCountByContractAddressByEpoch(
        transactions_count_by_contract_address_by_epoch::Reducer,
    ),
    TotalTransactionsCountByContractAddresses(
        total_transactions_count_by_contract_addresses::Reducer,
    ),
}

impl Plugin {
    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match self {
            Plugin::UtxoByAddress(x) => x.reduce_block(block, output),
            Plugin::PointByTx(x) => x.reduce_block(block, output),
            Plugin::PoolByStake(x) => x.reduce_block(block, output),
            Plugin::TotalTransactionsCount(x) => x.reduce_block(block, output),
            Plugin::TransactionsCountByEpoch(x) => x.reduce_block(block, output),
            Plugin::TransactionsCountByContractAddress(x) => x.reduce_block(block, output),
            Plugin::TransactionsCountByContractAddressByEpoch(x) => x.reduce_block(block, output),
            Plugin::TotalTransactionsCountByContractAddresses(x) => x.reduce_block(block, output),
        }
    }
}

pub struct Worker {
    input: InputPort,
    output: OutputPort,
    reducers: Vec<Plugin>,
    ops_count: gasket::metrics::Counter,
}

impl Worker {
    pub fn new(reducers: Vec<Plugin>) -> Self {
        Worker {
            reducers,
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
        }
    }

    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort {
        &mut self.output
    }

    pub fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        pipeline.register_stage("reducers", spawn_stage(self, Default::default()));
    }

    fn reduce_block(&mut self, block: &MultiEraBlock) -> Result<(), gasket::error::Error> {
        self.output.send(gasket::messaging::Message::from(
            CRDTCommand::block_starting(block),
        ))?;

        for reducer in self.reducers.iter_mut() {
            reducer.reduce_block(block, &mut self.output)?;
            self.ops_count.inc(1);
        }

        self.output.send(gasket::messaging::Message::from(
            CRDTCommand::block_finished(block),
        ))?;

        Ok(())
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
