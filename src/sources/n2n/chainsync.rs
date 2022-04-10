use std::cell::Cell;

use pallas::network::{
    miniprotocols::{self, chainsync, Error, Point},
    multiplexer::Channel,
};

use gasket::{
    error::AsWorkError,
    metrics::{Counter, Gauge},
};

use crate::model::ChainSyncCommand;

use super::messages;

struct ChainObserver {
    min_depth: usize,
    output: gasket::messaging::OutputPort<ChainSyncCommand>,
    chain_buffer: chainsync::RollbackBuffer,
    block_count: gasket::metrics::Counter,
    chain_tip: gasket::metrics::Gauge,
}

impl ChainObserver {
    fn new(
        min_depth: usize,
        block_count: Counter,
        chain_tip: Gauge,
        output: gasket::messaging::OutputPort<ChainSyncCommand>,
    ) -> Self {
        Self {
            min_depth,
            block_count,
            chain_tip,
            output,
            chain_buffer: Default::default(),
        }
    }
}

impl chainsync::Observer<chainsync::HeaderContent> for ChainObserver {
    fn on_roll_forward(
        &mut self,
        content: chainsync::HeaderContent,
        tip: &chainsync::Tip,
    ) -> Result<chainsync::Continuation, Error> {
        // parse the header and extract the point of the chain
        let header = messages::MultiEraHeader::try_from(content)?;
        let point = header.read_cursor()?;

        // track the new point in our memory buffer
        log::info!("rolling forward to point {:?}", point);
        self.chain_buffer.roll_forward(point);

        // see if we have points that already reached certain depth
        let ready = self.chain_buffer.pop_with_depth(self.min_depth);
        log::debug!("found {} points with required min depth", ready.len());

        // request download of blocks for confirmed points
        for point in ready {
            log::debug!("requesting block fetch for point {:?}", point);
            self.output
                .send(ChainSyncCommand::roll_forward(point.clone()))?;
            self.block_count.inc(1);

            // evaluate if we should finalize the thread according to config
            //if should_finalize(&self.finalize_config, &point,
            // self.block_count) {    return Ok(chainsync::
            // Continuation::DropOut);
            //}
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
                    .send(ChainSyncCommand::roll_back(point.clone()))?;
            }
        }

        Ok(chainsync::Continuation::Proceed)
    }
}

type OutputPort = gasket::messaging::OutputPort<ChainSyncCommand>;
type Runner = miniprotocols::Runner<chainsync::HeaderConsumer<ChainObserver>>;

pub struct Worker {
    channel: Channel,
    pub min_depth: usize,
    pub known_points: Option<Vec<Point>>,
    //finalize_config: Option<FinalizeConfig>,
    runner: Cell<Option<Runner>>,
    output: OutputPort,
    block_count: gasket::metrics::Counter,
    chain_tip: gasket::metrics::Gauge,
}

impl Worker {
    pub fn new(
        channel: Channel,
        min_depth: usize,
        known_points: Option<Vec<Point>>,
        output: OutputPort,
    ) -> Self {
        Self {
            channel,
            min_depth,
            known_points,
            output,
            runner: Cell::new(None),
            block_count: Default::default(),
            chain_tip: Default::default(),
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
        let mut runner = Runner::new(chainsync::Consumer::initial(
            self.known_points.clone(),
            ChainObserver::new(
                self.min_depth as usize,
                self.block_count.clone(),
                self.chain_tip.clone(),
                self.output.clone(),
            ),
        ));

        runner.start().or_work_err()?;

        self.runner.set(Some(runner));

        Ok(())
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        match self
            .runner
            .get_mut()
            .as_mut()
            .unwrap()
            .run_step(&mut self.channel)
        {
            Ok(true) => Ok(gasket::runtime::WorkOutcome::Done),
            Ok(false) => Ok(gasket::runtime::WorkOutcome::Partial),
            Err(err) => Err(gasket::error::Error::WorkError(format!(
                "chainsync agent error {:?}",
                err
            ))),
        }
    }
}
