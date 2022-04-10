pub mod redis;

use gasket::messaging::InputPort;

use crate::{bootstrap, model};

pub trait Pluggable {
    fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::CRDTCommand>;
    fn spawn(self, pipeline: &mut bootstrap::Pipeline);
}

pub enum Plugin {
    Redis(redis::Worker),
}

impl Plugin {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::CRDTCommand> {
        match self {
            Plugin::Redis(x) => x.borrow_input_port(),
        }
    }

    pub fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        match self {
            Plugin::Redis(x) => x.spawn(pipeline),
        }
    }
}
