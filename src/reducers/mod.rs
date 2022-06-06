use gasket::runtime::{spawn_stage, WorkOutcome};

use crate::{
    bootstrap,
    model::{self, CRDTCommand, MultiEraBlock},
};

type InputPort = gasket::messaging::InputPort<model::ChainSyncCommandEx>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

pub mod point_by_tx;
pub mod pool_by_stake;
pub mod utxo_by_address;

#[cfg(feature = "unstable")]
pub mod address_by_txo;
#[cfg(feature = "unstable")]
pub mod total_transactions_count;
#[cfg(feature = "unstable")]
pub mod total_transactions_count_by_contract_addresses;
#[cfg(feature = "unstable")]
pub mod transactions_count_by_contract_address;
#[cfg(feature = "unstable")]
pub mod transactions_count_by_contract_address_by_epoch;
#[cfg(feature = "unstable")]
pub mod transactions_count_by_epoch;

pub enum Plugin {
    UtxoByAddress(utxo_by_address::Reducer),
    PointByTx(point_by_tx::Reducer),
    PoolByStake(pool_by_stake::Reducer),

    #[cfg(feature = "unstable")]
    AddressByTxo(address_by_txo::Reducer),
    #[cfg(feature = "unstable")]
    TotalTransactionsCount(total_transactions_count::Reducer),
    #[cfg(feature = "unstable")]
    TransactionsCountByEpoch(transactions_count_by_epoch::Reducer),
    #[cfg(feature = "unstable")]
    TransactionsCountByContractAddress(transactions_count_by_contract_address::Reducer),
    #[cfg(feature = "unstable")]
    TransactionsCountByContractAddressByEpoch(
        transactions_count_by_contract_address_by_epoch::Reducer,
    ),
    #[cfg(feature = "unstable")]
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

            #[cfg(feature = "unstable")]
            Plugin::AddressByTxo(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Plugin::TotalTransactionsCount(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Plugin::TransactionsCountByEpoch(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Plugin::TransactionsCountByContractAddress(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Plugin::TransactionsCountByContractAddressByEpoch(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
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
