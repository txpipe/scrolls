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
use pallas::network::{
    miniprotocols::{chainsync::TipFinder, run_agent, Point},
    multiplexer::Channel,
};
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

    fn find_end_of_chain(&self, channel: &mut Channel) -> Result<Point, crate::Error> {
        let point = Point::Specific(
            self.chain.shelley_known_slot,
            hex::decode(&self.chain.shelley_known_hash)
                .map_err(|_| crate::Error::config("can't decode shelley known hash"))?,
        );

        let agent = TipFinder::initial(point);
        let agent = run_agent(agent, channel).map_err(crate::Error::ouroboros)?;

        match agent.output {
            Some(tip) => Ok(tip.0),
            None => Err(crate::Error::message("failure acquiring end of chain")),
        }
    }

    fn define_known_points(
        &self,
        channel: &mut Channel,
    ) -> Result<Option<Vec<Point>>, crate::Error> {
        match &self.intersect {
            crosscut::IntersectConfig::Origin => Ok(None),
            crosscut::IntersectConfig::Tip => {
                let tip = self.find_end_of_chain(channel)?;
                Ok(Some(vec![tip]))
            }
            crosscut::IntersectConfig::Point(x) => {
                let point = x.clone().try_into()?;
                Ok(Some(vec![point]))
            }
            crosscut::IntersectConfig::Fallbacks(x) => {
                let points: Result<Vec<_>, _> = x.iter().cloned().map(|x| x.try_into()).collect();
                Ok(Some(points?))
            }
        }
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

        let mut cs_channel = transport.muxer.use_channel(2);
        let bf_channel = transport.muxer.use_channel(3);

        let known_points = self
            .define_known_points(&mut cs_channel)
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

        super::Plugin::N2N(plugin)
    }
}
