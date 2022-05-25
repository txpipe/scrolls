pub mod chainsync;
mod transport;

use std::time::Duration;

use gasket::{error::AsWorkError, messaging::FanoutPort, retries};

use serde::Deserialize;

use crate::{bootstrap::Pipeline, crosscut, model::ChainSyncCommandEx};

use self::transport::Transport;

use super::utils;

#[derive(Deserialize)]
pub struct Config {
    pub path: String,
}

pub struct Plugin {
    config: Config,
    intersect: crosscut::IntersectConfig,
    chain: crosscut::ChainWellKnownInfo,
    output: FanoutPort<ChainSyncCommandEx>,
}

impl Plugin {
    fn bootstrap_transport(&self) -> Result<Transport, gasket::error::Error> {
        gasket::retries::retry_operation(
            || Transport::setup(&self.config.path, self.chain.magic).or_work_err(),
            &retries::Policy {
                max_retries: 5,
                backoff_factor: 2,
                backoff_unit: Duration::from_secs(1),
                max_backoff: Duration::from_secs(60),
            },
            None,
        )
    }
}

impl super::Pluggable for Plugin {
    fn borrow_output_port(&mut self) -> &'_ mut FanoutPort<ChainSyncCommandEx> {
        &mut self.output
    }

    fn spawn(self, pipeline: &mut Pipeline) {
        let mut transport = self
            .bootstrap_transport()
            .expect("transport should be connected after several retries");

        let mut cs_channel = transport.muxer.use_channel(5);

        let known_points =
            utils::define_known_points(&self.chain, &self.intersect, &mut cs_channel)
                .expect("chainsync known-points should be defined");

        pipeline.register_stage(
            "n2c",
            gasket::runtime::spawn_stage(
                self::chainsync::Worker::new(cs_channel, 0, known_points, self.output),
                gasket::runtime::Policy::default(),
            ),
        );
    }
}

impl super::IntoPlugin for Config {
    fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
    ) -> super::Plugin {
        let plugin = Plugin {
            config: self,
            intersect: intersect.clone(),
            chain: chain.clone(),
            output: Default::default(),
        };

        super::Plugin::N2C(plugin)
    }
}
