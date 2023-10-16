use pallas::{ledger::traverse::wellknown::GenesisValues, network::miniprotocols::Point};
use serde::Deserialize;

pub mod cursor;
pub mod errors;
pub mod model;

pub use cursor::*;
pub use errors::*;

use self::model::{RawBlockPayload, EnrichedBlockPayload};

#[derive(Debug, Clone)]
pub struct Record(pub Vec<u8>);

pub type SourceOutputPort = gasket::messaging::tokio::OutputPort<RawBlockPayload>;
pub type EnrichInputPort = gasket::messaging::tokio::InputPort<RawBlockPayload>;
pub type EnrichOutputPort = gasket::messaging::tokio::OutputPort<EnrichedBlockPayload>;
// pub type ReducerInputPort = gasket::messaging::tokio::InputPort<ChainEvent>;
// pub type ReducerOutputPort = gasket::messaging::tokio::OutputPort<ChainEvent>;
// pub type StorageInputPort = gasket::messaging::tokio::InputPort<ChainEvent>;
// pub type StorageOutputPort = gasket::messaging::tokio::OutputPort<ChainEvent>;

// TODO validate next implementation trait
pub type OutputAdapter = gasket::messaging::tokio::ChannelSendAdapter<RawBlockPayload>;
pub type InputAdapter = gasket::messaging::tokio::ChannelRecvAdapter<RawBlockPayload>;

pub trait StageBootstrapper {
    fn connect_output(&mut self, adapter: OutputAdapter);
    fn connect_input(&mut self, adapter: InputAdapter);
    fn spawn(self, policy: gasket::runtime::Policy) -> gasket::runtime::Tether;
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChainConfig {
    Mainnet,
    Testnet,
    PreProd,
    Preview,
    Custom(GenesisValues),
}
impl Default for ChainConfig {
    fn default() -> Self {
        Self::Mainnet
    }
}
impl From<ChainConfig> for GenesisValues {
    fn from(other: ChainConfig) -> Self {
        match other {
            ChainConfig::Mainnet => GenesisValues::mainnet(),
            ChainConfig::Testnet => GenesisValues::testnet(),
            ChainConfig::PreProd => GenesisValues::preprod(),
            ChainConfig::Preview => GenesisValues::preview(),
            ChainConfig::Custom(x) => x,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum IntersectConfig {
    Tip,
    Origin,
    Point(u64, String),
    Breadcrumbs(Vec<(u64, String)>),
}
impl IntersectConfig {
    pub fn points(&self) -> Option<Vec<Point>> {
        match self {
            IntersectConfig::Breadcrumbs(all) => {
                let mapped = all
                    .iter()
                    .map(|(slot, hash)| {
                        let hash = hex::decode(hash).expect("valid hex hash");
                        Point::Specific(*slot, hash)
                    })
                    .collect();

                Some(mapped)
            }
            IntersectConfig::Point(slot, hash) => {
                let hash = hex::decode(hash).expect("valid hex hash");
                Some(vec![Point::Specific(*slot, hash)])
            }
            _ => None,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct FinalizeConfig {
    until_hash: Option<String>,
    max_block_slot: Option<u64>,
}

pub struct Context {
    pub chain: ChainConfig,
    pub intersect: IntersectConfig,
    pub cursor: Cursor,
    pub finalize: Option<FinalizeConfig>,
}
