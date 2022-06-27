mod chainsync;
mod transport;

use serde::Deserialize;
use std::time::Duration;

use self::transport::Transport;
use crate::{bootstrap::Pipeline, crosscut, model::RawBlockPayload};
use gasket::{error::AsWorkError, messaging::OutputPort, retries};

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
    output: OutputPort<RawBlockPayload>,
}

impl Bootstrapper {
    fn bootstrap_transport(&self) -> Result<Transport, crate::Error> {
        gasket::retries::retry_operation(
            || Transport::setup(&self.config.path, self.chain.magic).or_retry(),
            &retries::Policy {
                max_retries: 5,
                backoff_factor: 2,
                backoff_unit: Duration::from_secs(1),
                max_backoff: Duration::from_secs(60),
            },
            None,
        )
        .map_err(crate::Error::source)
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort<RawBlockPayload> {
        &mut self.output
    }

    pub fn spawn_stages(self, pipeline: &mut Pipeline, cursor: &Option<crosscut::PointArg>) {
        let transport = self
            .bootstrap_transport()
            .expect("transport should be connected after several retries");

        pipeline.register_stage(
            "n2c",
            gasket::runtime::spawn_stage(
                self::chainsync::Worker::new(
                    transport.channel5,
                    0,
                    self.chain,
                    self.intersect,
                    cursor.clone(),
                    self.output,
                ),
                gasket::runtime::Policy {
                    tick_timeout: Some(Duration::from_secs(5)),
                    ..Default::default()
                },
            ),
        );
    }
}
