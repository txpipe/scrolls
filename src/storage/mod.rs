pub mod redis;

use gasket::messaging::FunnelPort;

use crate::{bootstrap, crosscut, model, Error};

pub trait Pluggable {
    fn borrow_input_port(&mut self) -> &'_ mut FunnelPort<model::CRDTCommand>;
    fn spawn(self, pipeline: &mut bootstrap::Pipeline);
    fn read_cursor(&self) -> Result<crosscut::Cursor, Error>;
}

pub enum Plugin {
    Redis(redis::Worker),
}

impl Plugin {
    pub fn borrow_input_port(&mut self) -> &'_ mut FunnelPort<model::CRDTCommand> {
        match self {
            Plugin::Redis(x) => x.borrow_input_port(),
        }
    }

    pub fn read_cursor(&self) -> Result<crosscut::Cursor, Error> {
        match self {
            Plugin::Redis(x) => x.read_cursor(),
        }
    }

    pub fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        match self {
            Plugin::Redis(x) => x.spawn(pipeline),
        }
    }
}
