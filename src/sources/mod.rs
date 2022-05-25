use gasket::messaging::FanoutPort;

use crate::{bootstrap, crosscut, model};

#[cfg(target_family = "unix")]
pub mod n2c;

pub mod n2n;
pub mod utils;

pub trait Pluggable {
    fn borrow_output_port(&mut self) -> &'_ mut FanoutPort<model::ChainSyncCommandEx>;
    fn spawn(self, pipeline: &mut bootstrap::Pipeline);
}

pub enum Plugin {
    N2N(n2n::Plugin),
    N2C(n2c::Plugin),
}

impl Plugin {
    pub fn borrow_output_port(&mut self) -> &'_ mut FanoutPort<model::ChainSyncCommandEx> {
        match self {
            Plugin::N2N(p) => p.borrow_output_port(),
            Plugin::N2C(p) => p.borrow_output_port(),
        }
    }

    pub fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        match self {
            Plugin::N2N(p) => p.spawn(pipeline),
            Plugin::N2C(p) => p.spawn(pipeline),
        }
    }
}

pub trait IntoPlugin {
    fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
    ) -> Plugin;
}
