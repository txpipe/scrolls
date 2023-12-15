use gasket::{messaging::RecvPort, runtime::Tether};
use serde::Deserialize;

use crate::framework::{errors::Error, *};

pub mod redis;

pub enum Bootstrapper {
    Redis(redis::Stage),
}

impl StageBootstrapper for Bootstrapper {
    fn connect_output(&mut self, _: OutputAdapter) {
        panic!("attempted to use storage stage as sender");
    }

    fn connect_input(&mut self, adapter: InputAdapter) {
        match self {
            Bootstrapper::Redis(p) => p.input.connect(adapter),
        }
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::Redis(s) => gasket::runtime::spawn_stage(s, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    Redis(redis::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::Redis(c) => Ok(Bootstrapper::Redis(c.bootstrapper(ctx)?)),
        }
    }
}
