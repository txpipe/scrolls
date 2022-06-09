use gasket::runtime::WorkOutcome;

use crate::{model, storage};

use super::Reducer;

type InputPort = gasket::messaging::InputPort<model::ChainSyncCommandEx>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

pub struct Worker {
    input: InputPort,
    output: OutputPort,
    reducers: Vec<Reducer>,
    state: storage::ReadPlugin,
    ops_count: gasket::metrics::Counter,
}

impl Worker {
    pub fn new(
        reducers: Vec<Reducer>,
        state: storage::ReadPlugin,
        input: InputPort,
        output: OutputPort,
    ) -> Self {
        Worker {
            reducers,
            state,
            input,
            output,
            ops_count: Default::default(),
        }
    }

    fn reduce_block(&mut self, block: &model::MultiEraBlock) -> Result<(), gasket::error::Error> {
        self.output.send(gasket::messaging::Message::from(
            model::CRDTCommand::block_starting(block),
        ))?;

        for reducer in self.reducers.iter_mut() {
            reducer.reduce_block(block, &mut self.output)?;
            self.ops_count.inc(1);
        }

        self.output.send(gasket::messaging::Message::from(
            model::CRDTCommand::block_finished(block),
        ))?;

        Ok(())
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv()?;

        match msg.payload {
            model::ChainSyncCommandEx::RollForward(block) => self.reduce_block(&block)?,
            model::ChainSyncCommandEx::RollBack(point) => {
                log::warn!("rollback requested for {:?}", point);
            }
        }

        Ok(WorkOutcome::Partial)
    }
}
