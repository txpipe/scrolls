mod chainsync;
mod transport;

use serde::Deserialize;
use std::time::Duration;

use crate::{bootstrap::Pipeline, crosscut, model::RawBlockPayload};
use gasket::messaging::OutputPort;

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
    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort<RawBlockPayload> {
        &mut self.output
    }

    pub fn spawn_stages(self, pipeline: &mut Pipeline, cursor: &Option<crosscut::PointArg>) {
        pipeline.register_stage(gasket::runtime::spawn_stage(
            self::chainsync::Worker::new(
                self.config.path.clone(),
                0,
                self.chain,
                self.intersect,
                cursor.clone(),
                self.output,
            ),
            gasket::runtime::Policy {
                tick_timeout: Some(Duration::from_secs(600)),
                bootstrap_retry: gasket::retries::Policy {
                    max_retries: 20,
                    backoff_factor: 2,
                    backoff_unit: Duration::from_secs(1),
                    max_backoff: Duration::from_secs(60),
                },
                ..Default::default()
            },
            Some("n2c"),
        ));
    }
}
