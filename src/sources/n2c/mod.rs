pub mod chainsync;
mod transport;

use std::time::Duration;

use gasket::{error::AsWorkError, messaging::OutputPort, retries};

use serde::Deserialize;

use crate::{bootstrap::Pipeline, crosscut, model::ChainSyncCommandEx, storage};

use self::transport::Transport;

#[derive(Deserialize)]
pub struct Config {
    pub path: String,
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
    output: OutputPort<ChainSyncCommandEx>,
}

impl Bootstrapper {
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

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort<ChainSyncCommandEx> {
        &mut self.output
    }

    pub fn spawn_stages(self, pipeline: &mut Pipeline, state: storage::ReadPlugin) {
        let mut transport = self
            .bootstrap_transport()
            .expect("transport should be connected after several retries");

        let cs_channel = transport.muxer.use_channel(5);

        pipeline.register_stage(
            "n2c",
            gasket::runtime::spawn_stage(
                self::chainsync::Worker::new(
                    cs_channel,
                    0,
                    self.chain,
                    self.intersect,
                    state,
                    self.output,
                ),
                gasket::runtime::Policy::default(),
            ),
        );
    }
}
