use gasket::{
    messaging::{RecvPort, SendPort},
    runtime::Tether,
};
use serde::Deserialize;

use crate::framework::*;

mod skip;
mod sled;

pub enum Bootstrapper {
    Skip(skip::Stage),
    Sled(sled::Stage),
}
impl StageBootstrapper for Bootstrapper {
    fn connect_output(&mut self, adapter: OutputAdapter) {
        match self {
            Bootstrapper::Skip(p) => p.output.connect(adapter),
            Bootstrapper::Sled(p) => p.output.connect(adapter),
        }
    }

    fn connect_input(&mut self, adapter: InputAdapter) {
        match self {
            Bootstrapper::Skip(p) => p.input.connect(adapter),
            Bootstrapper::Sled(p) => p.input.connect(adapter),
        }
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Skip(s) => gasket::runtime::spawn_stage(s, policy),
            Bootstrapper::Sled(s) => gasket::runtime::spawn_stage(s, policy),
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
