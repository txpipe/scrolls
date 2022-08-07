pub mod redis;
pub mod skip;

use gasket::messaging::InputPort;
use serde::Deserialize;

use crate::{
    bootstrap,
    crosscut::{self, PointArg},
    model,
};

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Redis(redis::Config),
    Skip(skip::Config),
}

impl Config {
    pub fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
    ) -> Bootstrapper {
        match self {
            Config::Redis(c) => Bootstrapper::Redis(c.boostrapper(chain, intersect)),
            Config::Skip(c) => Bootstrapper::Skip(c.boostrapper()),
        }
    }
}

pub enum Bootstrapper {
    Redis(redis::Bootstrapper),
    Skip(skip::Bootstrapper),
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::CRDTCommand> {
        match self {
            Bootstrapper::Redis(x) => x.borrow_input_port(),
            Bootstrapper::Skip(x) => x.borrow_input_port(),
        }
    }

    pub fn build_cursor(&mut self) -> Cursor {
        match self {
            Bootstrapper::Redis(x) => Cursor::Redis(x.build_cursor()),
            Bootstrapper::Skip(x) => Cursor::Skip(x.build_cursor()),
        }
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        match self {
            Bootstrapper::Redis(x) => x.spawn_stages(pipeline),
            Bootstrapper::Skip(x) => x.spawn_stages(pipeline),
        }
    }
}

pub enum Cursor {
    Redis(redis::Cursor),
    Skip(skip::Cursor),
}

impl Cursor {
    pub fn last_point(&mut self) -> Result<Option<PointArg>, crate::Error> {
        match self {
            Cursor::Redis(x) => x.last_point(),
            Cursor::Skip(x) => x.last_point(),
        }
    }
}
