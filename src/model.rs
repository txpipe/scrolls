use std::collections::HashSet;

use pallas::{ledger::traverse::MultiEraBlock, network::miniprotocols::Point};

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
    RollForward(Vec<u8>),
    RollBack(Point),
}

impl ChainSyncCommandEx {
    pub fn roll_forward(block: Vec<u8>) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollForward(block),
        }
    }

    pub fn roll_back(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollBack(point),
        }
    }
}

pub type Set = String;
pub type Member = String;
pub type Key = String;
pub type Value = String;
pub type Delta = i64;
pub type Timestamp = u64;

#[derive(Debug)]
#[non_exhaustive]
pub enum CRDTCommand {
    BlockStarting(Point),
    TwoPhaseSetAdd(Set, Member),
    TwoPhaseSetRemove(Set, Member),
    GrowOnlySetAdd(Set, Member),
    LastWriteWins(Key, Value, Timestamp),
    AnyWriteWins(Key, Value),
    // TODO make sure Value is a generic not stringly typed
    PNCounter(Key, Delta),
    BlockFinished(Point),
}

impl CRDTCommand {
    pub fn block_starting(block: &MultiEraBlock) -> CRDTCommand {
        let hash = block.hash();
        let slot = block.slot();
        let point = Point::Specific(slot, hash.to_vec());
        CRDTCommand::BlockStarting(point)
    }

    pub fn block_finished(block: &MultiEraBlock) -> CRDTCommand {
        let hash = block.hash();
        let slot = block.slot();
        let point = Point::Specific(slot, hash.to_vec());
        CRDTCommand::BlockFinished(point)
    }
}

pub enum StateQuery {
    KeyValue(Key),
    LatestKeyValue(Key),
    SetMembers(Set),
}

pub enum StateData {
    KeyValue(Value),
    SetMembers(HashSet<Member>),
}
