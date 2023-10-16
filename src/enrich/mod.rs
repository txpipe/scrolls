use gasket::{messaging::RecvPort, runtime::Tether};
use serde::Deserialize;

use crate::framework::*;

mod skip;
mod sled;

pub enum Bootstrapper {
    Skip(skip::Stage),
    Sled(sled::Stage),
}
impl StageBootstrapper for Bootstrapper {
    fn connect_output(&mut self, _: OutputAdapter) {
        panic!("attempted to use enrich stage as sender");
    }

    fn connect_input(&mut self, adapter: InputAdapter) {
        match self {
            Bootstrapper::Skip(p) => p.input.connect(adapter),
            Bootstrapper::Sled(p) => p.input.connect(adapter),
        }
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Skip(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::Sled(x) => gasket::runtime::spawn_stage(x, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Skip(skip::Config),
    Sled(sled::Config),
}
impl Default for Config {
    fn default() -> Self {
        Self::Skip(skip::Config::default())
    }
}
impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Skip(c) => Ok(Bootstrapper::Skip(c.bootstrapper(ctx)?)),
            Config::Sled(c) => Ok(Bootstrapper::Sled(c.bootstrapper(ctx)?)),
        }
    }
}
