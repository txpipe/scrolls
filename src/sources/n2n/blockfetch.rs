use pallas::network::{
    miniprotocols::{blockfetch, run_agent, Error, Point},
    multiplexer::Channel,
};

use gasket::{error::*, runtime::WorkOutcome};

use crate::model::{ChainSyncCommand, ChainSyncCommandEx};

use crate::sources::utils;

struct Observer<'a> {
    output: &'a mut OutputPort,
}

impl<'a> blockfetch::Observer for Observer<'a> {
    fn on_block_received(&mut self, body: Vec<u8>) -> Result<(), Error> {
        let block = utils::parse_block_content(&body)?;
        self.output.send(ChainSyncCommandEx::roll_forward(block))?;

        Ok(())
    }
}

pub type InputPort = gasket::messaging::InputPort<ChainSyncCommand>;
pub type OutputPort = gasket::messaging::OutputPort<ChainSyncCommandEx>;

pub struct Worker {
    channel: Channel,
    block_count: gasket::metrics::Counter,
    input: InputPort,
    output: OutputPort,
}

impl Worker {
    pub fn new(channel: Channel, input: InputPort, output: OutputPort) -> Self {
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
        run_agent(agent, &mut self.channel)?;

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
        let input = self.input.recv()?;

        match input.payload {
            ChainSyncCommand::RollForward(point) => {
                self.fetch_block(point).or_work_err()?;
            }
            ChainSyncCommand::RollBack(point) => {
                self.output.send(ChainSyncCommandEx::roll_back(point))?;
            }
        };

        Ok(WorkOutcome::Partial)
    }
}
