use gasket::{messaging::SendPort, runtime::Tether};
use serde::Deserialize;

use crate::framework::{errors::Error, *};

pub mod n2c;
pub mod n2n;
pub mod ogmios;
pub mod hydra;
pub mod hydra_udp;
pub mod hydra_ws;

pub enum Bootstrapper {
    N2N(n2n::Stage),
    N2C(n2c::Stage),
    Ogmios(ogmios::Stage),
    HydraUdp(hydra_udp::Stage),
    HydraWs(hydra_ws::Stage),
}

impl StageBootstrapper for Bootstrapper {
    fn connect_output(&mut self, adapter: OutputAdapter) {
        match self {
            Bootstrapper::N2N(p) => p.output.connect(adapter),
            Bootstrapper::N2C(p) => p.output.connect(adapter),
            Bootstrapper::Ogmios(p) => p.output.connect(adapter),
            Bootstrapper::HydraUdp(p) => p.output.connect(adapter),
            Bootstrapper::HydraWs(p) => p.output.connect(adapter),
        }
    }

    fn connect_input(&mut self, _: InputAdapter) {
        panic!("attempted to use source stage as receiver");
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        match self {
            Bootstrapper::N2N(s) => gasket::runtime::spawn_stage(s, policy),
            Bootstrapper::N2C(s) => gasket::runtime::spawn_stage(s, policy),
            Bootstrapper::Ogmios(s) => gasket::runtime::spawn_stage(s, policy),
            Bootstrapper::HydraUdp(s) => gasket::runtime::spawn_stage(s, policy),
            Bootstrapper::HydraWs(s) => gasket::runtime::spawn_stage(s, policy),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    N2N(n2n::Config),

    #[cfg(target_family = "unix")]
    N2C(n2c::Config),

    Ogmios(ogmios::Config),

    HydraUdp(hydra_udp::Config),

    HydraWs(hydra_ws::Config),
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Bootstrapper, Error> {
        match self {
            Config::N2N(c) => Ok(Bootstrapper::N2N(c.bootstrapper(ctx)?)),
            Config::N2C(c) => Ok(Bootstrapper::N2C(c.bootstrapper(ctx)?)),
            Config::Ogmios(c) => Ok(Bootstrapper::Ogmios(c.bootstrapper(ctx)?)),
            Config::HydraUdp(c) => Ok(Bootstrapper::HydraUdp(c.bootstrapper(ctx)?)),
            Config::HydraWs(c) => Ok(Bootstrapper::HydraWs(c.bootstrapper(ctx)?)),
        }
    }
}
