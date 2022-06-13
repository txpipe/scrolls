pub mod sled;

use gasket::messaging::{InputPort, OutputPort};
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Sled(sled::Config),
}

impl Config {
    pub fn Bootstrapper(self) -> Bootstrapper {
        match self {
            Config::Sled(c) => Bootstrapper::Sled(c.boostrapper()),
        }
    }
}

pub enum Bootstrapper {
    Sled(sled::Bootstrapper),
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::ChainSyncCommandEx> {
        match self {
            Bootstrapper::Sled(x) => x.borrow_input_port(),
        }
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort<model::ChainSyncCommandEx> {
        match self {
            Bootstrapper::Sled(x) => x.borrow_output_port(),
        }
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        match self {
            Bootstrapper::Sled(x) => x.spawn_stages(pipeline),
        }
    }
}
