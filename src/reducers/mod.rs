use gasket::{
    messaging::{RecvPort, SendPort},
    runtime::Tether,
};
use serde::Deserialize;

use crate::framework::{errors::Error, *};

pub mod builtin;

#[cfg(feature = "deno")]
pub mod deno;

pub enum Bootstrapper {
    BuiltIn(builtin::Stage),

    #[cfg(feature = "deno")]
    Deno(deno::Stage),
}

impl StageBootstrapper for Bootstrapper {
    fn connect_output(&mut self, adapter: OutputAdapter) {
        match self {
            Bootstrapper::BuiltIn(p) => p.output.connect(adapter),

            #[cfg(feature = "deno")]
            Bootstrapper::Deno(p) => p.output.connect(adapter),
        }
    }

    fn connect_input(&mut self, adapter: InputAdapter) {
        match self {
            Bootstrapper::BuiltIn(p) => p.input.connect(adapter),

            #[cfg(feature = "deno")]
            Bootstrapper::Deno(p) => p.input.connect(adapter),
        }
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::BuiltIn(s) => gasket::runtime::spawn_stage(s, policy),

            #[cfg(feature = "deno")]
            Bootstrapper::Deno(s) => gasket::runtime::spawn_stage(s, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    BuiltIn(builtin::Config),

    #[cfg(feature = "deno")]
    Deno(deno::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::BuiltIn(c) => Ok(Bootstrapper::BuiltIn(c.bootstrapper(ctx)?)),

            #[cfg(feature = "deno")]
            Config::Deno(c) => Ok(Bootstrapper::Deno(c.bootstrapper(ctx)?)),
        }
    }
}
