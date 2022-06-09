use gasket::runtime::spawn_stage;
use serde::Deserialize;

use crate::{bootstrap, crosscut, model, storage};

type InputPort = gasket::messaging::InputPort<model::ChainSyncCommandEx>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

pub mod point_by_tx;
pub mod pool_by_stake;
pub mod utxo_by_address;
mod worker;

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

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    UtxoByAddress(utxo_by_address::Config),
    PointByTx(point_by_tx::Config),
    PoolByStake(pool_by_stake::Config),

    #[cfg(feature = "unstable")]
    AddressByTxo(address_by_txo::Config),
    #[cfg(feature = "unstable")]
    TotalTransactionsCount(total_transactions_count::Config),
    #[cfg(feature = "unstable")]
    TransactionsCountByEpoch(transactions_count_by_epoch::Config),
    #[cfg(feature = "unstable")]
    TransactionsCountByContractAddress(transactions_count_by_contract_address::Config),
    #[cfg(feature = "unstable")]
    TransactionsCountByContractAddressByEpoch(
        transactions_count_by_contract_address_by_epoch::Config,
    ),
    #[cfg(feature = "unstable")]
    TotalTransactionsCountByContractAddresses(
        total_transactions_count_by_contract_addresses::Config,
    ),
}

impl Config {
    fn plugin(self, chain: &crosscut::ChainWellKnownInfo) -> Reducer {
        match self {
            Config::UtxoByAddress(c) => c.plugin(chain),
            Config::PointByTx(c) => c.plugin(),
            Config::PoolByStake(c) => c.plugin(),

            #[cfg(feature = "unstable")]
            Config::AddressByTxo(c) => c.plugin(chain),
            #[cfg(feature = "unstable")]
            Config::TotalTransactionsCount(c) => c.plugin(),
            #[cfg(feature = "unstable")]
            Config::TransactionsCountByEpoch(c) => c.plugin(chain),
            #[cfg(feature = "unstable")]
            Config::TransactionsCountByContractAddress(c) => c.plugin(chain),
            #[cfg(feature = "unstable")]
            Config::TransactionsCountByContractAddressByEpoch(c) => c.plugin(chain),
            #[cfg(feature = "unstable")]
            Config::TotalTransactionsCountByContractAddresses(c) => c.plugin(),
        }
    }
}

pub struct Bootstrapper {
    input: InputPort,
    output: OutputPort,
    reducers: Vec<Reducer>,
}

impl Bootstrapper {
    pub fn new(configs: Vec<Config>, chain: &crosscut::ChainWellKnownInfo) -> Self {
        Self {
            reducers: configs.into_iter().map(|x| x.plugin(&chain)).collect(),
            input: Default::default(),
            output: Default::default(),
        }
    }

    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort {
        &mut self.output
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline, state: storage::ReadPlugin) {
        let worker = worker::Worker::new(self.reducers, state, self.input, self.output);
        pipeline.register_stage("reducers", spawn_stage(worker, Default::default()));
    }
}

pub enum Reducer {
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

impl Reducer {
    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match self {
            Reducer::UtxoByAddress(x) => x.reduce_block(block, output),
            Reducer::PointByTx(x) => x.reduce_block(block, output),
            Reducer::PoolByStake(x) => x.reduce_block(block, output),

            #[cfg(feature = "unstable")]
            Reducer::AddressByTxo(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::TotalTransactionsCount(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::TransactionsCountByEpoch(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::TransactionsCountByContractAddress(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::TransactionsCountByContractAddressByEpoch(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::TotalTransactionsCountByContractAddresses(x) => x.reduce_block(block, output),
        }
    }
}
