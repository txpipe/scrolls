use std::time::Duration;

use gasket::runtime::spawn_stage;
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

type InputPort = gasket::messaging::TwoPhaseInputPort<model::EnrichedBlockPayload>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

pub mod macros;
pub mod point_by_tx;
pub mod pool_by_stake;
pub mod utxo_by_address;
mod worker;

#[cfg(feature = "unstable")]
pub mod address_by_ada_handle;
#[cfg(feature = "unstable")]
pub mod address_by_txo;
#[cfg(feature = "unstable")]
pub mod asset_holders_by_asset_id;
#[cfg(feature = "unstable")]
pub mod balance_by_address;
#[cfg(feature = "unstable")]
pub mod block_header_by_hash;
#[cfg(feature = "unstable")]
pub mod last_block_parameters;
#[cfg(feature = "unstable")]
pub mod tx_by_hash;
#[cfg(feature = "unstable")]
pub mod tx_count_by_address;
#[cfg(feature = "unstable")]
pub mod tx_count_by_native_token_policy_id;
#[cfg(feature = "unstable")]
pub mod utxos_by_asset;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    UtxoByAddress(utxo_by_address::Config),
    PointByTx(point_by_tx::Config),
    PoolByStake(pool_by_stake::Config),

    #[cfg(feature = "unstable")]
    AddressByTxo(address_by_txo::Config),
    #[cfg(feature = "unstable")]
    BalanceByAddress(balance_by_address::Config),
    #[cfg(feature = "unstable")]
    TxByHash(tx_by_hash::Config),
    #[cfg(feature = "unstable")]
    TxCountByAddress(tx_count_by_address::Config),
    #[cfg(feature = "unstable")]
    BlockHeaderByHash(block_header_by_hash::Config),
    #[cfg(feature = "unstable")]
    AddressByAdaHandle(address_by_ada_handle::Config),
    #[cfg(feature = "unstable")]
    LastBlockParameters(last_block_parameters::Config),
    #[cfg(feature = "unstable")]
    TxCountByNativeTokenPolicyId(tx_count_by_native_token_policy_id::Config),
    #[cfg(feature = "unstable")]
    AssetHoldersByAsset(asset_holders_by_asset_id::Config),
    #[cfg(feature = "unstable")]
    UtxosByAsset(utxos_by_asset::Config),
}

impl Config {
    fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        policy: &crosscut::policies::RuntimePolicy,
    ) -> Reducer {
        match self {
            Config::UtxoByAddress(c) => c.plugin(policy),
            Config::PointByTx(c) => c.plugin(),
            Config::PoolByStake(c) => c.plugin(),

            #[cfg(feature = "unstable")]
            Config::AddressByTxo(c) => c.plugin(policy),
            #[cfg(feature = "unstable")]
            Config::BalanceByAddress(c) => c.plugin(policy),
            #[cfg(feature = "unstable")]
            Config::TxByHash(c) => c.plugin(policy),
            #[cfg(feature = "unstable")]
            Config::TxCountByAddress(c) => c.plugin(policy),
            #[cfg(feature = "unstable")]
            Config::BlockHeaderByHash(c) => c.plugin(policy),
            #[cfg(feature = "unstable")]
            Config::AddressByAdaHandle(c) => c.plugin(),
            #[cfg(feature = "unstable")]
            Config::LastBlockParameters(c) => c.plugin(chain),
            #[cfg(feature = "unstable")]
            Config::TxCountByNativeTokenPolicyId(c) => c.plugin(chain),
            #[cfg(feature = "unstable")]
            Config::AssetHoldersByAsset(c) => c.plugin(chain, policy),
            #[cfg(feature = "unstable")]
            Config::UtxosByAsset(c) => c.plugin(policy),
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
        pipeline.register_stage(spawn_stage(
            worker,
            gasket::runtime::Policy {
                tick_timeout: Some(Duration::from_secs(600)),
                ..Default::default()
            },
            Some("reducers"),
        ));
    }
}

pub enum Reducer {
    UtxoByAddress(utxo_by_address::Reducer),
    PointByTx(point_by_tx::Reducer),
    PoolByStake(pool_by_stake::Reducer),

    #[cfg(feature = "unstable")]
    AddressByTxo(address_by_txo::Reducer),
    #[cfg(feature = "unstable")]
    BalanceByAddress(balance_by_address::Reducer),
    #[cfg(feature = "unstable")]
    TxByHash(tx_by_hash::Reducer),
    #[cfg(feature = "unstable")]
    TxCountByAddress(tx_count_by_address::Reducer),
    #[cfg(feature = "unstable")]
    BlockHeaderByHash(block_header_by_hash::Reducer),
    #[cfg(feature = "unstable")]
    AddressByAdaHandle(address_by_ada_handle::Reducer),
    #[cfg(feature = "unstable")]
    LastBlockParameters(last_block_parameters::Reducer),
    #[cfg(feature = "unstable")]
    TxCountByNativeTokenPolicyId(tx_count_by_native_token_policy_id::Reducer),
    #[cfg(feature = "unstable")]
    AssetHoldersByAssetId(asset_holders_by_asset_id::Reducer),
    #[cfg(feature = "unstable")]
    UtxosByAsset(utxos_by_asset::Reducer),
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
            Reducer::AddressByTxo(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::BalanceByAddress(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::TxByHash(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::TxCountByAddress(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::BlockHeaderByHash(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::AddressByAdaHandle(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::LastBlockParameters(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::TxCountByNativeTokenPolicyId(x) => x.reduce_block(block, output),
            #[cfg(feature = "unstable")]
            Reducer::AssetHoldersByAssetId(x) => x.reduce_block(block, ctx, output),
            #[cfg(feature = "unstable")]
            Reducer::UtxosByAsset(x) => x.reduce_block(block, ctx, output),
        }
    }
}
