use std::{ops::Deref, sync::Arc};

use pallas::{
    ledger::primitives::{alonzo, byron},
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

impl MultiEraBlock {
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
                let hash = alonzo::crypto::hash_block_header(&x.1.header);
                Ok(Point::Specific(x.1.header.header_body.slot, hash.to_vec()))
            }
        }
    }
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
    // TODO make sure Value is a generic not stringly typed
    PNCounter(Key, Value),
}
