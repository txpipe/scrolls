use pallas::{ledger::traverse::wellknown::GenesisValues, network::miniprotocols::Point};
use serde::Deserialize;

use self::cursor::Cursor;

pub mod cursor;
pub mod errors;
pub mod model;

pub struct Record(pub Vec<u8>);

pub enum ChainEvent {
    Apply(Point, Record),
    Undo(Point, Record),
    Reset(Point),
}
impl ChainEvent {
    pub fn apply(point: Point, record: impl Into<Record>) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::Apply(point, record.into()),
        }
    }

    pub fn undo(point: Point, record: impl Into<Record>) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::Undo(point, record.into()),
        }
    }

    pub fn reset(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::Reset(point),
        }
    }

    pub fn point(&self) -> &Point {
        match self {
            Self::Apply(x, _) => x,
            Self::Undo(x, _) => x,
            Self::Reset(x) => x,
        }
    }

    pub fn record(&self) -> Option<&Record> {
        match self {
            Self::Apply(_, x) => Some(x),
            Self::Undo(_, x) => Some(x),
            _ => None,
        }
    }

    pub fn map_record(self, f: fn(Record) -> Record) -> Self {
        match self {
            Self::Apply(p, x) => Self::Apply(p, f(x)),
            Self::Undo(p, x) => Self::Undo(p, f(x)),
            Self::Reset(x) => Self::Reset(x),
        }
    }

    pub fn try_map_record<E>(self, f: fn(Record) -> Result<Record, E>) -> Result<Self, E> {
        let out = match self {
            Self::Apply(p, x) => Self::Apply(p, f(x)?),
            Self::Undo(p, x) => Self::Undo(p, f(x)?),
            Self::Reset(x) => Self::Reset(x),
        };

        Ok(out)
    }

    pub fn try_map_record_to_many<E>(
        self,
        f: fn(Record) -> Result<Vec<Record>, E>,
    ) -> Result<Vec<Self>, E> {
        let out = match self {
            Self::Apply(p, x) => f(x)?
                .into_iter()
                .map(|i| Self::Apply(p.clone(), i))
                .collect(),
            Self::Undo(p, x) => f(x)?
                .into_iter()
                .map(|i| Self::Undo(p.clone(), i))
                .collect(),
            Self::Reset(x) => vec![Self::Reset(x)],
        };

        Ok(out)
    }
}

pub type SourceOutputPort = gasket::messaging::tokio::OutputPort<ChainEvent>;
pub type EnrichInputPort = gasket::messaging::tokio::InputPort<ChainEvent>;
pub type EnrichOutputPort = gasket::messaging::tokio::OutputPort<ChainEvent>;
pub type ReducerInputPort = gasket::messaging::tokio::InputPort<ChainEvent>;
pub type ReducerOutputPort = gasket::messaging::tokio::OutputPort<ChainEvent>;
pub type StorageInputPort = gasket::messaging::tokio::InputPort<ChainEvent>;
pub type StorageOutputPort = gasket::messaging::tokio::OutputPort<ChainEvent>;

pub type OutputAdapter = gasket::messaging::tokio::ChannelSendAdapter<ChainEvent>;
pub type InputAdapter = gasket::messaging::tokio::ChannelRecvAdapter<ChainEvent>;

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
