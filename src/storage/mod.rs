pub mod redis;

use gasket::messaging::InputPort;
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Redis(redis::Config),
}

impl Config {
    pub fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
    ) -> Bootstrapper {
        match self {
            Config::Redis(c) => Bootstrapper::Redis(c.boostrapper(chain, intersect)),
        }
    }
}

pub enum Bootstrapper {
    Redis(redis::Bootstrapper),
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort<model::CRDTCommand> {
        match self {
            Bootstrapper::Redis(x) => x.borrow_input_port(),
        }
    }

    pub fn build_read_plugin(&self) -> ReadPlugin {
        match self {
            Bootstrapper::Redis(x) => ReadPlugin::Redis(x.build_read_plugin()),
        }
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        match self {
            Bootstrapper::Redis(x) => x.spawn_stages(pipeline),
        }
    }
}

pub enum ReadPlugin {
    Redis(redis::ReadPlugin),
}

impl ReadPlugin {
    pub fn bootstrap(&mut self) -> Result<(), crate::Error> {
        match self {
            ReadPlugin::Redis(x) => x.bootstrap(),
        }
    }

    pub fn read_state(
        &mut self,
        query: model::StateQuery,
    ) -> Result<model::StateData, crate::Error> {
        match self {
            ReadPlugin::Redis(x) => x.read_state(query),
        }
    }

    pub fn read_cursor(&mut self) -> Result<crosscut::Cursor, crate::Error> {
        match self {
            ReadPlugin::Redis(x) => x.read_cursor(),
        }
    }
}
