pub mod skip;
pub mod sled;

use gasket::messaging::{InputPort, OutputPort};
use serde::Deserialize;

use crate::{bootstrap, model};

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Skip,
    Sled(sled::Config),
}

impl Config {
    pub fn bootstrapper(self) -> Bootstrapper {
        match self {
            Config::Skip => Bootstrapper::Skip(skip::Bootstrapper::default()),
            Config::Sled(c) => Bootstrapper::Sled(c.boostrapper()),
        }
    }
}

pub enum Bootstrapper {
    Skip(skip::Bootstrapper),
    Sled(sled::Bootstrapper),
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::RawBlockPayload> {
        match self {
            Bootstrapper::Skip(x) => x.borrow_input_port(),
            Bootstrapper::Sled(x) => x.borrow_input_port(),
        }
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort<model::EnrichedBlockPayload> {
        match self {
            Bootstrapper::Skip(x) => x.borrow_output_port(),
            Bootstrapper::Sled(x) => x.borrow_output_port(),
        }
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        match self {
            Bootstrapper::Skip(x) => x.spawn_stages(pipeline),
            Bootstrapper::Sled(x) => x.spawn_stages(pipeline),
        }
    }
}
