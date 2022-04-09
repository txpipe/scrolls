use pallas::{
    ledger::primitives::{alonzo, byron, probing, Era, Fragment},
    network::{
        miniprotocols::{blockfetch, run_agent, Error, Point},
        multiplexer::Channel,
    },
};

use gasket::{error::*, runtime::WorkOutcome};

use crate::model::MultiEraBlock;

use super::{messages, ChainSyncCommandEx};

struct Observer<'a> {
    output: &'a mut OutputPort,
}

impl<'a> blockfetch::Observer for Observer<'a> {
    fn on_block_received(&mut self, body: Vec<u8>) -> Result<(), Error> {
        let block = match probing::probe_block_cbor_era(&body) {
            probing::Outcome::Matched(era) => match era {
                Era::Byron => MultiEraBlock::Byron(byron::Block::decode_fragment(&body)?),
                _ => MultiEraBlock::AlonzoCompatible(alonzo::BlockWrapper::decode_fragment(&body)?),
            },
            // TODO: we're assuming that the genesis block is Byron-compatible. Is this a safe
            // assumption?
            probing::Outcome::GenesisBlock => {
                MultiEraBlock::Byron(byron::Block::decode_fragment(&body)?)
            }
            probing::Outcome::Inconclusive => {
                let msg = format!("can't infer primitive block from cbor, inconclusive probing. CBOR hex for debugging: {}", hex::encode(body));
                return Err(msg.into());
            }
        };

        self.output.send(ChainSyncCommandEx::roll_forward(block))?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct Config {}

pub type InputPort = gasket::messaging::InputPort<messages::ChainSyncCommand>;
pub type OutputPort = gasket::messaging::OutputPort<messages::ChainSyncCommandEx>;

pub struct Worker {
    config: Config,
    channel: Channel,
    block_count: gasket::metrics::Counter,
    pub input: InputPort,
    pub output: OutputPort,
}

impl Worker {
    pub fn new(channel: Channel, config: Config) -> Self {
        Self {
            config,
            channel,
            input: Default::default(),
            output: Default::default(),
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
            super::ChainSyncCommand::RollForward(point) => {
                self.fetch_block(point).or_work_err()?;
            }
            super::ChainSyncCommand::RollBack(point) => {
                self.output.send(ChainSyncCommandEx::roll_back(point))?;
            }
        };

        Ok(WorkOutcome::Partial)
    }
}
