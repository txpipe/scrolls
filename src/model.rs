use pallas::ledger::primitives::{alonzo, byron};

#[derive(Debug)]
pub enum MultiEraBlock {
    AlonzoCompatible(alonzo::BlockWrapper),
    Byron(byron::Block),
}

pub type Set = String;
pub type Member = String;

#[derive(Debug)]
pub enum CRDTCommand {
    TwoPhaseSetAdd(Set, Member),
    TwoPhaseSetRemove(Set, Member),
}
