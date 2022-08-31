use std::{collections::HashMap, ops::Deref};

use pallas::ledger::traverse::MultiEraBlock;
use pallas::network::miniprotocols::{self, chainsync, Agent, Point};

use gasket::{
    error::AsWorkError,
    metrics::{Counter, Gauge},
};

use crate::{crosscut, model, sources::utils, storage};

use super::transport::Transport;

struct ChainObserver {
    min_depth: usize,
    output: OutputPort,
    chain_buffer: chainsync::RollbackBuffer,
    blocks: HashMap<Point, Vec<u8>>,
    block_count: Counter,
    chain_tip: Gauge,
    finalize_config: Option<crosscut::FinalizeConfig>,
}

impl ChainObserver {
    fn new(min_depth: usize, 
        block_count: Counter,
        chain_tip: Gauge,
        output: OutputPort,
        finalize_config: Option<crosscut::FinalizeConfig>,
    ) -> Self {
        Self {
            min_depth,
            block_count,
            chain_tip,
            output,
            chain_buffer: Default::default(),
            blocks: Default::default(),
            finalize_config
        }
    }
}

impl chainsync::Observer<chainsync::BlockContent> for ChainObserver {
    fn on_roll_forward(
        &mut self,
        content: chainsync::BlockContent,
        tip: &chainsync::Tip,
    ) -> Result<chainsync::Continuation, Box<dyn std::error::Error>> {
        // parse the block and extract the point of the chain
        let cbor = Vec::from(content.deref());
        let block = MultiEraBlock::decode(&cbor)?;
        let point = Point::Specific(block.slot(), block.hash().to_vec());

        // store the block for later retrieval
        self.blocks.insert(point.clone(), cbor);

        // track the new point in our memory buffer
        log::debug!("rolling forward to point {:?}", point);
        self.chain_buffer.roll_forward(point);

        // see if we have points that already reached certain depth
        let ready = self.chain_buffer.pop_with_depth(self.min_depth);
        log::debug!("found {} points with required min depth", ready.len());

        // find confirmed block in memory and send down the pipeline
        for point in ready {
            let block = self
                .blocks
                .remove(&point)
                .expect("required block not found in memory");

            self.output
                .send(model::RawBlockPayload::roll_forward(block))?;
            self.block_count.inc(1);
            
            // evaluate if we should finalize the thread according to config
            if crosscut::should_finalize(&self.finalize_config, &point) {
                return Ok(chainsync::Continuation::DropOut);
            }
        }

        // notify chain tip to the pipeline metrics
        self.chain_tip.set(tip.1 as i64);

        Ok(chainsync::Continuation::Proceed)
    }

    fn on_rollback(
        &mut self,
        point: &Point,
    ) -> Result<chainsync::Continuation, Box<dyn std::error::Error>> {
        log::debug!("rolling block to point {:?}", point);

        match self.chain_buffer.roll_back(point) {
            chainsync::RollbackEffect::Handled => {
                log::debug!("handled rollback within buffer {:?}", point);
            }
            chainsync::RollbackEffect::OutOfScope => {
                log::debug!("rollback out of buffer scope, sending event down the pipeline");
                self.output
                    .send(model::RawBlockPayload::roll_back(point.clone()))?;
            }
        }

        Ok(chainsync::Continuation::Proceed)
    }
}

type OutputPort = gasket::messaging::OutputPort<model::RawBlockPayload>;
type MyAgent = chainsync::BlockConsumer<ChainObserver>;

pub struct Worker {
    socket: String,
    min_depth: usize,
    chain: crosscut::ChainWellKnownInfo,
    intersect: crosscut::IntersectConfig,
    finalize: Option<crosscut::FinalizeConfig>,
    cursor: storage::Cursor,
    agent: Option<MyAgent>,
    transport: Option<Transport>,
    output: OutputPort,
    block_count: gasket::metrics::Counter,
    chain_tip: gasket::metrics::Gauge,
}

impl Worker {
    pub fn new(
        socket: String,
        min_depth: usize,
        chain: crosscut::ChainWellKnownInfo,
        intersect: crosscut::IntersectConfig,
        finalize: Option<crosscut::FinalizeConfig>,
        cursor: storage::Cursor,
        output: OutputPort,
    ) -> Self {
        Self {
            socket,
            min_depth,
            chain,
            intersect,
            finalize,
            cursor,
            output,
            agent: None,
            transport: None,
            block_count: Default::default(),
            chain_tip: Default::default(),
        }
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("received_blocks", &self.block_count)
            .with_gauge("chain_tip", &self.chain_tip)
            .build()
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let mut transport = Transport::setup(&self.socket, self.chain.magic).or_retry()?;

        let known_points = utils::define_chainsync_start(
            &self.chain,
            &self.intersect,
            &mut self.cursor,
            &mut transport.channel5,
        )
        .or_retry()?;

        let agent = MyAgent::initial(
            known_points,
            ChainObserver::new(
                self.min_depth,
                self.block_count.clone(),
                self.chain_tip.clone(),
                self.output.clone(),
                self.finalize.clone(),
            ),
        )
        .apply_start()
        .or_retry()?;

        self.agent = Some(agent);
        self.transport = Some(transport);

        Ok(())
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let agent = self.agent.take().unwrap();
        let mut transport = self.transport.take().unwrap();

        let agent = miniprotocols::run_agent_step(agent, &mut transport.channel5).or_restart()?;

        let is_done = agent.is_done();

        self.agent = Some(agent);
        self.transport = Some(transport);

        match is_done {
            true => Ok(gasket::runtime::WorkOutcome::Done),
            false => Ok(gasket::runtime::WorkOutcome::Partial),
        }
    }
}
