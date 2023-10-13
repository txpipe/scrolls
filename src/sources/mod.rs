use gasket::{messaging::SendPort, runtime::Tether};
use serde::Deserialize;

use crate::framework::{*, errors::Error};

pub mod n2c;
pub mod n2n;

pub enum Bootstrapper {
    N2N(n2n::Stage),
    N2C(n2c::Stage),
}

impl StageBootstrapper for Bootstrapper {
    fn connect_output(&mut self, adapter: OutputAdapter) {
        match self {
            Bootstrapper::N2N(p) => p.output.connect(adapter),
            Bootstrapper::N2C(p) => p.output.connect(adapter),
        }
    }

    fn connect_input(&mut self, _: InputAdapter) {
        panic!("attempted to use source stage as receiver");
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::N2N(x) => gasket::runtime::spawn_stage(x, policy),
            Bootstrapper::N2C(x) => gasket::runtime::spawn_stage(x, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    N2N(n2n::Config),

    #[cfg(target_family = "unix")]
    N2C(n2c::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::N2N(c) => Ok(Bootstrapper::N2N(c.bootstrapper(ctx)?)),
            Config::N2C(c) => Ok(Bootstrapper::N2C(c.bootstrapper(ctx)?)),
        }
    }
}
