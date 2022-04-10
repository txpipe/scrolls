use gasket::messaging::{InputPort, OutputPort};

use crate::{bootstrap, model};

pub mod utxo_by_address;

pub trait Pluggable {
    fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::ChainSyncCommandEx>;
    fn borrow_output_port(&mut self) -> &'_ mut OutputPort<model::CRDTCommand>;
    fn spawn(self, pipeline: &mut bootstrap::Pipeline);
}

pub enum Plugin {
    UtxoByAddress(utxo_by_address::Worker),
}

impl Plugin {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::ChainSyncCommandEx> {
        match self {
            Plugin::UtxoByAddress(x) => x.borrow_input_port(),
        }
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort<model::CRDTCommand> {
        match self {
            Plugin::UtxoByAddress(x) => x.borrow_output_port(),
        }
    }

    pub fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        match self {
            Plugin::UtxoByAddress(x) => x.spawn(pipeline),
        }
    }
}
