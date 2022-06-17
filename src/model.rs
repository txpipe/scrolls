use std::collections::{HashMap, HashSet};

use pallas::{
    crypto::hash::Hash,
    ledger::traverse::{Era, MultiEraBlock, MultiEraInput, MultiEraTx},
    network::miniprotocols::Point,
};

use crate::Error;

#[derive(Debug, Clone)]
pub enum RawBlockPayload {
    RollForward(Vec<u8>),
    RollBack(Point),
}

impl RawBlockPayload {
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

#[derive(Default, Debug, Clone)]
pub struct BlockContext {
    ref_txs: HashMap<String, (Era, Vec<u8>)>,
}

impl BlockContext {
    pub fn set_ref_tx(&mut self, hash: &Hash<32>, era: Era, cbor: Vec<u8>) {
        self.ref_txs.insert(hash.to_string(), (era, cbor));
    }

    pub fn decode_ref_tx(&self, input: &MultiEraInput) -> Result<Option<MultiEraTx>, Error> {
        if let Some(output_ref) = input.output_ref() {
            if let Some((era, cbor)) = self.ref_txs.get(&output_ref.tx_id().to_string()) {
                let tx = MultiEraTx::decode(*era, &cbor).map_err(crate::Error::cbor)?;
                return Ok(Some(tx));
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub enum EnrichedBlockPayload {
    RollForward(Vec<u8>, BlockContext),
    RollBack(Point),
}

impl EnrichedBlockPayload {
    pub fn roll_forward(block: Vec<u8>, ctx: BlockContext) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollForward(block, ctx),
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
    NotFound,
    KeyValue(Value),
    SetMembers(HashSet<Member>),
}

impl From<Option<Value>> for StateData {
    fn from(maybe: Option<Value>) -> Self {
        match maybe {
            Some(x) => StateData::KeyValue(x),
            None => StateData::NotFound,
        }
    }
}

impl From<Option<HashSet<Value>>> for StateData {
    fn from(maybe: Option<HashSet<Value>>) -> Self {
        match maybe {
            Some(x) => StateData::SetMembers(x),
            None => StateData::NotFound,
        }
    }
}
