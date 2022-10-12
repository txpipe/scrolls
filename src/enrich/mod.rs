pub mod skip;
pub mod sled;

use gasket::messaging::{OutputPort, TwoPhaseInputPort};
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Skip,
    Sled(sled::Config),
}

impl Default for Config {
    fn default() -> Self {
        Self::Skip
    }
}

impl Config {
    pub fn bootstrapper(self, policy: &crosscut::policies::RuntimePolicy) -> Bootstrapper {
        match self {
            Config::Skip => Bootstrapper::Skip(skip::Bootstrapper::default()),
            Config::Sled(c) => Bootstrapper::Sled(c.boostrapper(policy)),
        }
    }
}

pub enum Bootstrapper {
    Skip(skip::Bootstrapper),
    Sled(sled::Bootstrapper),
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut TwoPhaseInputPort<model::RawBlockPayload> {
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
