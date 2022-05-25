use std::{collections::HashMap, ops::Deref};

use pallas::network::{
    miniprotocols::{self, chainsync, Error, Point},
    multiplexer::Channel,
};

use gasket::{
    error::AsWorkError,
    metrics::{Counter, Gauge},
};

use crate::{
    model::{ChainSyncCommandEx, MultiEraBlock},
    sources::utils,
};

struct ChainObserver {
    min_depth: usize,
    output: gasket::messaging::FanoutPort<ChainSyncCommandEx>,
    chain_buffer: chainsync::RollbackBuffer,
    blocks: HashMap<Point, MultiEraBlock>,
    block_count: gasket::metrics::Counter,
    chain_tip: Gauge,
}

impl ChainObserver {
    fn new(
        min_depth: usize,
        block_count: Counter,
        chain_tip: Gauge,
        output: gasket::messaging::FanoutPort<ChainSyncCommandEx>,
    ) -> Self {
        Self {
            min_depth,
            block_count,
            chain_tip,
            output,
            chain_buffer: Default::default(),
            blocks: Default::default(),
        }
    }
}

impl chainsync::Observer<chainsync::BlockContent> for ChainObserver {
    fn on_roll_forward(
        &mut self,
        content: chainsync::BlockContent,
        tip: &chainsync::Tip,
    ) -> Result<chainsync::Continuation, Error> {
        // parse the block and extract the point of the chain
        let cbor = Vec::from(content.deref());
        let block = utils::parse_block_content(&cbor)?;
        let point = block.point()?;

        // store the block for later retrieval
        self.blocks.insert(point.clone(), block);

        // track the new point in our memory buffer
        log::info!("rolling forward to point {:?}", point);
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

            self.output.send(ChainSyncCommandEx::roll_forward(block))?;
            self.block_count.inc(1);
        }

        // notify chain tip to the pipeline metrics
        self.chain_tip.set(tip.1 as i64);

        Ok(chainsync::Continuation::Proceed)
    }

    fn on_rollback(&mut self, point: &Point) -> Result<chainsync::Continuation, Error> {
        log::info!("rolling block to point {:?}", point);

        match self.chain_buffer.roll_back(point) {
            chainsync::RollbackEffect::Handled => {
                log::debug!("handled rollback within buffer {:?}", point);
            }
            chainsync::RollbackEffect::OutOfScope => {
                log::debug!("rollback out of buffer scope, sending event down the pipeline");
                self.output
                    .send(ChainSyncCommandEx::roll_back(point.clone()))?;
            }
        }

        Ok(chainsync::Continuation::Proceed)
    }
}

type OutputPort = gasket::messaging::FanoutPort<ChainSyncCommandEx>;
type Runner = miniprotocols::Runner<chainsync::BlockConsumer<ChainObserver>>;

pub struct Worker {
    channel: Channel,
    pub min_depth: usize,
    pub known_points: Option<Vec<Point>>,
    //finalize_config: Option<FinalizeConfig>,
    runner: Runner,
    block_count: gasket::metrics::Counter,
    chain_tip: Gauge,
}

impl Worker {
    pub fn new(
        channel: Channel,
        min_depth: usize,
        known_points: Option<Vec<Point>>,
        output: OutputPort,
    ) -> Self {
        let block_count = Counter::default();
        let chain_tip = Gauge::default();

        let runner = Runner::new(chainsync::Consumer::initial(
            known_points.clone(),
            ChainObserver::new(
                min_depth as usize,
                block_count.clone(),
                chain_tip.clone(),
                output,
            ),
        ));

        Self {
            channel,
            min_depth,
            known_points,
            runner,
            block_count,
            chain_tip,
        }
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("block_count", &self.block_count)
            .with_gauge("chain_tip", &self.chain_tip)
            .build()
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        self.runner.start().or_work_err()?;

        Ok(())
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        match self.runner.run_step(&mut self.channel) {
            Ok(true) => Ok(gasket::runtime::WorkOutcome::Done),
            Ok(false) => Ok(gasket::runtime::WorkOutcome::Partial),
            Err(err) => Err(gasket::error::Error::WorkError(format!(
                "chainsync agent error {:?}",
                err
            ))),
        }
    }
}
