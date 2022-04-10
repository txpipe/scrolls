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
use serde::Deserialize;

use crate::{
    bootstrap::Pipeline,
    crosscut,
    model::{ChainSyncCommand, ChainSyncCommandEx},
};

use self::transport::Transport;

#[derive(Deserialize)]
pub struct Config {
    pub address: String,

    #[serde(deserialize_with = "crate::crosscut::deserialize_magic_arg")]
    pub magic: Option<crosscut::MagicArg>,
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
        let magic = self.config.magic.unwrap_or_default();

        let mut transport = gasket::retries::retry_operation(
            || Transport::setup(&self.config.address, *magic).or_work_err(),
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
                self::chainsync::Worker::new(transport.muxer.use_channel(2), 0, None, headers_out),
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
