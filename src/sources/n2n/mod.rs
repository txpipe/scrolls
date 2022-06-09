pub mod blockfetch;
pub mod chainsync;
mod messages;
mod transport;

use std::time::Duration;

use gasket::{
    error::AsWorkError,
    messaging::{FanoutPort, InputPort, OutputPort},
    retries,
};
pub use messages::*;

use serde::Deserialize;

use crate::{
    bootstrap::Pipeline,
    crosscut,
    model::{ChainSyncCommand, ChainSyncCommandEx},
    storage,
};

use self::transport::Transport;

use super::utils;

#[derive(Deserialize)]
pub struct Config {
    pub address: String,
}

impl Config {
    pub fn bootstrapper(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        intersect: &crosscut::IntersectConfig,
    ) -> Bootstrapper {
        Bootstrapper {
            config: self,
            intersect: intersect.clone(),
            chain: chain.clone(),
            output: Default::default(),
        }
    }
}

pub struct Bootstrapper {
    config: Config,
    intersect: crosscut::IntersectConfig,
    chain: crosscut::ChainWellKnownInfo,
    output: FanoutPort<ChainSyncCommandEx>,
}

impl Bootstrapper {
    fn bootstrap_transport(&self) -> Result<Transport, gasket::error::Error> {
        gasket::retries::retry_operation(
            || Transport::setup(&self.config.address, self.chain.magic).or_work_err(),
            &retries::Policy {
                max_retries: 5,
                backoff_factor: 2,
                backoff_unit: Duration::from_secs(1),
                max_backoff: Duration::from_secs(60),
            },
            None,
        )
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut FanoutPort<ChainSyncCommandEx> {
        &mut self.output
    }

    pub fn spawn(self, pipeline: &mut Pipeline, storage: &storage::ReadPlugin) {
        let mut transport = self
            .bootstrap_transport()
            .expect("transport should be connected after several retries");

        let mut cs_channel = transport.muxer.use_channel(2);
        let bf_channel = transport.muxer.use_channel(3);

        let known_points =
            utils::define_known_points(&self.chain, &self.intersect, storage, &mut cs_channel)
                .expect("chainsync known-points should be defined");

        let mut headers_out = OutputPort::<ChainSyncCommand>::default();
        let mut headers_in = InputPort::<ChainSyncCommand>::default();
        gasket::messaging::connect_ports(&mut headers_out, &mut headers_in, 10);

        pipeline.register_stage(
            "n2n-headers",
            gasket::runtime::spawn_stage(
                self::chainsync::Worker::new(cs_channel, 0, known_points, headers_out),
                gasket::runtime::Policy::default(),
            ),
        );

        pipeline.register_stage(
            "n2n-blocks",
            gasket::runtime::spawn_stage(
                self::blockfetch::Worker::new(bf_channel, headers_in, self.output),
                gasket::runtime::Policy::default(),
            ),
        );
    }
}
