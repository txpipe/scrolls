use std::sync::Arc;

use pallas::{
    ledger::primitives::{alonzo, byron},
    network::miniprotocols::Point,
};

#[derive(Debug)]
pub enum ChainSyncCommand {
    RollForward(Point),
    RollBack(Point),
}

impl ChainSyncCommand {
    pub fn roll_forward(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollForward(point),
        }
    }

    pub fn roll_back(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollBack(point),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChainSyncCommandEx {
    RollForward(Arc<MultiEraBlock>),
    RollBack(Point),
}

impl ChainSyncCommandEx {
    pub fn roll_forward(block: MultiEraBlock) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollForward(Arc::new(block)),
        }
    }

    pub fn roll_back(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollBack(point),
        }
    }
}

#[derive(Debug)]
pub enum MultiEraBlock {
    AlonzoCompatible(alonzo::BlockWrapper),
    Byron(byron::Block),
}

pub type Set = String;
pub type Member = String;
pub type Key = String;
pub type Value = String;
pub type Timestamp = u64;

#[derive(Debug)]
pub enum CRDTCommand {
    TwoPhaseSetAdd(Set, Member),
    TwoPhaseSetRemove(Set, Member),
    GrowOnlySetAdd(Set, Member),
    LastWriteWins(Key, Value, Timestamp),
}
