use std::time::Duration;

use gasket::runtime::spawn_stage;
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

type InputPort = gasket::messaging::InputPort<model::EnrichedBlockPayload>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

pub mod point_by_tx;
pub mod pool_by_stake;
pub mod utxo_by_address;
mod worker;

#[cfg(feature = "unstable")]
pub mod address_by_txo;
#[cfg(feature = "unstable")]
pub mod balance_by_address;
#[cfg(feature = "unstable")]
pub mod total_transactions_count;
#[cfg(feature = "unstable")]
pub mod total_transactions_count_by_addresses;
#[cfg(feature = "unstable")]
pub mod transactions_count_by_address;
#[cfg(feature = "unstable")]
pub mod transactions_count_by_address_by_epoch;
#[cfg(feature = "unstable")]
pub mod transactions_count_by_epoch;
#[cfg(feature = "unstable")]
pub mod transactions_count_by_script_hash;

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
    TransactionsCountByAddress(transactions_count_by_address::Config),
    #[cfg(feature = "unstable")]
    TransactionsCountByAddressByEpoch(transactions_count_by_address_by_epoch::Config),
    #[cfg(feature = "unstable")]
    TotalTransactionsCountByAddresses(total_transactions_count_by_addresses::Config),
    #[cfg(feature = "unstable")]
    BalanceByAddress(balance_by_address::Config),
    #[cfg(feature = "unstable")]
    TransactionsCountByScriptHash(transactions_count_by_script_hash::Config),
}

impl Config {
    fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        policy: &crosscut::policies::RuntimePolicy,
    ) -> Reducer {
        match self {
            Config::UtxoByAddress(c) => c.plugin(chain, policy),
            Config::PointByTx(c) => c.plugin(),
            Config::PoolByStake(c) => c.plugin(),

            #[cfg(feature = "unstable")]
            Config::AddressByTxo(c) => c.plugin(chain),
            #[cfg(feature = "unstable")]
            Config::TotalTransactionsCount(c) => c.plugin(),
            #[cfg(feature = "unstable")]
            Config::TransactionsCountByEpoch(c) => c.plugin(chain),
            #[cfg(feature = "unstable")]
            Config::TransactionsCountByAddress(c) => c.plugin(chain, policy),
            #[cfg(feature = "unstable")]
            Config::TransactionsCountByAddressByEpoch(c) => c.plugin(chain, policy),
            #[cfg(feature = "unstable")]
            Config::TotalTransactionsCountByAddresses(c) => c.plugin(),
            #[cfg(feature = "unstable")]
            Config::BalanceByAddress(c) => c.plugin(chain, policy),
            #[cfg(feature = "unstable")]
            Config::TransactionsCountByScriptHash(c) => c.plugin(),
        }
    }
}

pub struct Bootstrapper {
    input: InputPort,
    output: OutputPort,
    reducers: Vec<Reducer>,
    policy: crosscut::policies::RuntimePolicy,
}

impl Bootstrapper {
    pub fn new(
        configs: Vec<Config>,
        chain: &crosscut::ChainWellKnownInfo,
        policy: &crosscut::policies::RuntimePolicy,
    ) -> Self {
        Self {
            reducers: configs
                .into_iter()
                .map(|x| x.plugin(chain, policy))
                .collect(),
            input: Default::default(),
            output: Default::default(),
            policy: policy.clone(),
        }
    }

    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort {
        &mut self.output
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        let worker = worker::Worker::new(self.reducers, self.input, self.output, self.policy);
        pipeline.register_stage(
            "reducers",
            spawn_stage(
                worker,
                gasket::runtime::Policy {
                    tick_timeout: Some(Duration::from_secs(5)),
                    ..Default::default()
                },
            ),
        );
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
    TransactionsCountByAddress(transactions_count_by_address::Reducer),
    #[cfg(feature = "unstable")]
    TransactionsCountByAddressByEpoch(transactions_count_by_address_by_epoch::Reducer),
    #[cfg(feature = "unstable")]
    TotalTransactionsCountByAddresses(total_transactions_count_by_addresses::Reducer),
    #[cfg(feature = "unstable")]
    BalanceByAddress(balance_by_address::Reducer),
    #[cfg(feature = "unstable")]
    TransactionsCountByScriptHash(transactions_count_by_script_hash::Reducer),
}

impl Reducer {
    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match self {
            Reducer::UtxoByAddress(x) => x.reduce_block(block, ctx, output),
            Reducer::PointByTx(x) => x.reduce_block(block, output),
            Reducer::PoolByStake(x) => x.reduce_block(block, output),

            #[cfg(feature = "unstable")]
            Reducer::AddressByTxo(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::TotalTransactionsCount(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::TransactionsCountByEpoch(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::TransactionsCountByAddress(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::TransactionsCountByAddressByEpoch(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::TotalTransactionsCountByAddresses(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::BalanceByAddress(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::TransactionsCountByScriptHash(x) => x.reduce_block(block, output),
        }
    }
}
