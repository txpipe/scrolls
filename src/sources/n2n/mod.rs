pub mod blockfetch;
pub mod chainsync;
mod transport;

use std::time::Duration;

use gasket::{
    error::AsWorkError,
    messaging::{InputPort, OutputPort},
    retries,
};

use pallas::network::miniprotocols::Point;
use serde::Deserialize;

use crate::{bootstrap::Pipeline, crosscut, model::RawBlockPayload};

use self::transport::Transport;

#[derive(Debug)]
pub enum ChainSyncInternalPayload {
    RollForward(Point),
    RollBack(Point),
}

impl ChainSyncInternalPayload {
    pub fn roll_forward(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollForward(point),
        }
    }

    pub fn roll_back(point: Point) -> gasket::messaging::Message<Self> {
        gasket::messaging::Message {
            payload: Self::RollBack(point),
        }
    }
}

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
    output: OutputPort<RawBlockPayload>,
}

impl Bootstrapper {
    fn bootstrap_transport(&self) -> Result<Transport, crate::Error> {
        gasket::retries::retry_operation(
            || Transport::setup(&self.config.address, self.chain.magic).or_retry(),
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

        let mut headers_out = OutputPort::<ChainSyncInternalPayload>::default();
        let mut headers_in = InputPort::<ChainSyncInternalPayload>::default();
        gasket::messaging::connect_ports(&mut headers_out, &mut headers_in, 10);

        pipeline.register_stage(
            "n2n-headers",
            gasket::runtime::spawn_stage(
                self::chainsync::Worker::new(
                    transport.channel2,
                    0,
                    self.chain,
                    self.intersect,
                    cursor.clone(),
                    headers_out,
                ),
                gasket::runtime::Policy {
                    tick_timeout: Some(Duration::from_secs(5)),
                    ..Default::default()
                },
            ),
        );

        pipeline.register_stage(
            "n2n-blocks",
            gasket::runtime::spawn_stage(
                self::blockfetch::Worker::new(transport.channel3, headers_in, self.output),
                gasket::runtime::Policy {
                    tick_timeout: Some(Duration::from_secs(5)),
                    ..Default::default()
                },
            ),
        );
    }
}
