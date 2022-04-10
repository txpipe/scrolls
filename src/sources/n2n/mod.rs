pub mod blockfetch;
pub mod chainsync;
mod messages;
mod transport;

use std::time::Duration;

use gasket::{
    error::AsWorkError,
    messaging::{InputPort, OutputPort},
    retries,
};
pub use messages::*;
use pallas::network::miniprotocols::Point;

use crate::{
    bootstrap::Pipeline,
    model::{ChainSyncCommand, ChainSyncCommandEx},
};

use self::transport::Transport;

pub struct Config {
    pub address: String,
    pub magic: u64,
}

pub struct Plugin {
    config: Config,
    output: OutputPort<ChainSyncCommandEx>,
}

impl super::Pluggable for Plugin {
    fn borrow_output_port(&mut self) -> &'_ mut gasket::messaging::OutputPort<ChainSyncCommandEx> {
        &mut self.output
    }

    fn spawn(self, pipeline: &mut Pipeline) {
        let byron_point = Point::Specific(
            43159,
            hex::decode("f5d398d6f71a9578521b05c43a668b06b6103f94fcf8d844d4c0aa906704b7a6")
                .unwrap(),
        );

        let alonzo_point = Point::Specific(
            57867490,
            hex::decode("c491c5006192de2c55a95fb3544f60b96bd1665accaf2dfa2ab12fc7191f016b")
                .unwrap(),
        );

        let mut transport = gasket::retries::retry_operation(
            || Transport::setup(&self.config.address, self.config.magic).or_work_err(),
            &retries::Policy {
                max_retries: 5,
                backoff_factor: 2,
                backoff_unit: Duration::from_secs(1),
                max_backoff: Duration::from_secs(60),
            },
            None,
        )
        .expect("connected transport after several retries");

        let mut headers_out = OutputPort::<ChainSyncCommand>::default();
        let mut headers_in = InputPort::<ChainSyncCommand>::default();
        gasket::messaging::connect_ports(&mut headers_out, &mut headers_in, 10);

        pipeline.register_stage(
            "n2n-headers",
            gasket::runtime::spawn_stage(
                self::chainsync::Worker::new(
                    transport.muxer.use_channel(2),
                    0,
                    Some(vec![alonzo_point]),
                    headers_out,
                ),
                gasket::runtime::Policy::default(),
            ),
        );

        pipeline.register_stage(
            "n2n-blocks",
            gasket::runtime::spawn_stage(
                self::blockfetch::Worker::new(
                    transport.muxer.use_channel(3),
                    headers_in,
                    self.output,
                ),
                gasket::runtime::Policy::default(),
            ),
        );
    }
}

impl From<Config> for Plugin {
    fn from(other: Config) -> Self {
        Plugin {
            config: other,
            output: Default::default(),
        }
    }
}
