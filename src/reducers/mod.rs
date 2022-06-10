use gasket::runtime::spawn_stage;
use serde::Deserialize;

use crate::{bootstrap, crosscut, model, storage};

type InputPort = gasket::messaging::InputPort<model::ChainSyncCommandEx>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

pub mod point_by_tx;
pub mod pool_by_stake;
pub mod utxo_by_address;
mod worker;

pub mod address_by_txo;
pub mod plutus_script_by_hash;
pub mod total_transactions_count;
pub mod total_transactions_count_by_contract_addresses;
pub mod transactions_count_by_contract_address;
pub mod transactions_count_by_contract_address_by_epoch;
pub mod transactions_count_by_epoch;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    UtxoByAddress(utxo_by_address::Config),
    PointByTx(point_by_tx::Config),
    PoolByStake(pool_by_stake::Config),

    
    AddressByTxo(address_by_txo::Config),
    
    PlutusScriptByHash(plutus_script_by_hash::Config),
    
    TotalTransactionsCount(total_transactions_count::Config),
    
    TransactionsCountByEpoch(transactions_count_by_epoch::Config),
    
    TransactionsCountByContractAddress(transactions_count_by_contract_address::Config),
    
    TransactionsCountByContractAddressByEpoch(
        transactions_count_by_contract_address_by_epoch::Config,
    ),
    
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

            
            Config::AddressByTxo(c) => c.plugin(chain),
            
            Config::PlutusScriptByHash(c) => c.plugin(chain),
            
            Config::TotalTransactionsCount(c) => c.plugin(),
            
            Config::TransactionsCountByEpoch(c) => c.plugin(chain),
            
            Config::TransactionsCountByContractAddress(c) => c.plugin(chain),
            
            Config::TransactionsCountByContractAddressByEpoch(c) => c.plugin(chain),
            
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

    
    AddressByTxo(address_by_txo::Reducer),
    
    PlutusScriptByHash(plutus_script_by_hash::Reducer),
    
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

            
            Reducer::AddressByTxo(x) => x.reduce_block(block, output),
            
            Reducer::PlutusScriptByHash(x) => x.reduce_block(block, output),
            
            Reducer::TotalTransactionsCount(x) => x.reduce_block(block, output),
            
            Reducer::TransactionsCountByEpoch(x) => x.reduce_block(block, output),
            
            Reducer::TransactionsCountByContractAddress(x) => x.reduce_block(block, output),
            
            Reducer::TransactionsCountByContractAddressByEpoch(x) => x.reduce_block(block, output),
            
            Reducer::TotalTransactionsCountByContractAddresses(x) => x.reduce_block(block, output),
        }
    }
}
