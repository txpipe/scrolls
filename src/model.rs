use std::{collections::HashSet, ops::Deref};

use pallas::{
    ledger::primitives::{alonzo, byron, probing, Era, Fragment, ToHash},
    network::miniprotocols::Point,
};

use crate::Error;

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

#[derive(Debug)]
pub enum MultiEraBlock<'b> {
    AlonzoCompatible(alonzo::BlockWrapper<'b>),
    Byron(byron::Block),
}

pub fn parse_block_content(body: &[u8]) -> Result<MultiEraBlock, Error> {
    match probing::probe_block_cbor_era(&body) {
        probing::Outcome::Matched(era) => match era {
            Era::Byron => {
                let primitive = byron::Block::decode_fragment(&body)?;
                let block = MultiEraBlock::Byron(primitive);
                Ok(block)
            }
            _ => {
                let primitive = alonzo::BlockWrapper::decode_fragment(&body)?;
                let block = MultiEraBlock::AlonzoCompatible(primitive);
                Ok(block)
            }
        },
        // TODO: we're assuming that the genesis block is Byron-compatible. Is this a safe
        // assumption?
        probing::Outcome::GenesisBlock => {
            let primitive = byron::Block::decode_fragment(&body)?;
            let block = MultiEraBlock::Byron(primitive);
            Ok(block)
        }
        probing::Outcome::Inconclusive => {
            let msg = format!("can't infer primitive block from cbor, inconclusive probing. CBOR hex for debugging: {}", hex::encode(body));
            return Err(Error::Message(msg));
        }
    }
}

impl<'b> MultiEraBlock<'b> {
    pub fn point(&self) -> Result<Point, Error> {
        match self {
            MultiEraBlock::Byron(x) => match x.deref() {
                byron::Block::EbBlock(x) => {
                    let hash = x.header.to_hash();
                    let slot = x.header.to_abs_slot();
                    Ok(Point::Specific(slot, hash.to_vec()))
                }
                byron::Block::MainBlock(x) => {
                    let hash = x.header.to_hash();
                    let slot = x.header.consensus_data.0.to_abs_slot();
                    Ok(Point::Specific(slot, hash.to_vec()))
                }
            },
            MultiEraBlock::AlonzoCompatible(x) => {
                let hash = &x.1.header.to_hash();
                Ok(Point::Specific(x.1.header.header_body.slot, hash.to_vec()))
            }
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
        let point = block.point().expect("block has defined point");
        CRDTCommand::BlockStarting(point)
    }

    pub fn block_finished(block: &MultiEraBlock) -> CRDTCommand {
        let point = block.point().expect("block has defined point");
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
