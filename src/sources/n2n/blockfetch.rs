use pallas::network::{
    miniprotocols::{blockfetch, run_agent, Point},
    multiplexer,
};

use gasket::{error::*, runtime::WorkOutcome};

use super::ChainSyncInternalPayload;
use crate::model::RawBlockPayload;

struct Observer<'a> {
    output: &'a mut OutputPort,
}

impl<'a> blockfetch::Observer for Observer<'a> {
    fn on_block_received(&mut self, body: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        self.output.send(RawBlockPayload::roll_forward(body))?;

        Ok(())
    }
}

pub type InputPort = gasket::messaging::InputPort<ChainSyncInternalPayload>;
pub type OutputPort = gasket::messaging::OutputPort<RawBlockPayload>;

pub struct Worker {
    channel: multiplexer::StdChannelBuffer,
    block_count: gasket::metrics::Counter,
    input: InputPort,
    output: OutputPort,
}

impl Worker {
    pub fn new(
        channel: multiplexer::StdChannelBuffer,
        input: InputPort,
        output: OutputPort,
    ) -> Self {
        Self {
            channel,
            input,
            output,
            block_count: Default::default(),
        }
    }

    fn fetch_block(&mut self, point: Point) -> Result<(), Error> {
        log::debug!("initiating chainsync");

        let observer = Observer {
            output: &mut self.output,
        };

        let agent = blockfetch::BatchClient::initial((point.clone(), point), observer);
        run_agent(agent, &mut self.channel).or_restart()?;

        self.block_count.inc(1);

        Ok(())
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("block_count", &self.block_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let input = self.input.recv_or_idle()?;

        match input.payload {
            ChainSyncInternalPayload::RollForward(point) => {
                self.fetch_block(point)?;
            }
            ChainSyncInternalPayload::RollBack(point) => {
                self.output.send(RawBlockPayload::roll_back(point))?;
            }
        };

        Ok(WorkOutcome::Partial)
    }
}
