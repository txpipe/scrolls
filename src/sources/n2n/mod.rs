pub mod blockfetch;
pub mod chainsync;
mod transport;

use std::time::Duration;

use gasket::messaging::{InputPort, OutputPort};

use pallas::network::miniprotocols::Point;
use serde::Deserialize;

use crate::{bootstrap::Pipeline, crosscut, model::RawBlockPayload};

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
    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort<RawBlockPayload> {
        &mut self.output
    }

    pub fn spawn_stages(self, pipeline: &mut Pipeline, cursor: &Option<crosscut::PointArg>) {
        let mut headers_out = OutputPort::<ChainSyncInternalPayload>::default();
        let mut headers_in = InputPort::<ChainSyncInternalPayload>::default();
        gasket::messaging::connect_ports(&mut headers_out, &mut headers_in, 10);

        pipeline.register_stage(gasket::runtime::spawn_stage(
            self::chainsync::Worker::new(
                self.config.address.clone(),
                0,
                self.chain.clone(),
                self.intersect,
                cursor.clone(),
                headers_out,
            ),
            gasket::runtime::Policy {
                tick_timeout: Some(Duration::from_secs(600)),
                ..Default::default()
            },
            Some("n2n-headers"),
        ));

        pipeline.register_stage(gasket::runtime::spawn_stage(
            self::blockfetch::Worker::new(
                self.config.address.clone(),
                self.chain,
                headers_in,
                self.output,
            ),
            gasket::runtime::Policy {
                tick_timeout: Some(Duration::from_secs(600)),
                ..Default::default()
            },
            Some("n2n-blocks"),
        ));
    }
}
