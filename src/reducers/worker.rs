use pallas::ledger::traverse::MultiEraBlock;

use crate::{crosscut, model, prelude::*};

use super::Reducer;

type InputPort = gasket::messaging::InputPort<model::EnrichedBlockPayload>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

pub struct Worker {
    input: InputPort,
    output: OutputPort,
    reducers: Vec<Reducer>,
    policy: crosscut::policies::RuntimePolicy,
    ops_count: gasket::metrics::Counter,
}

impl Worker {
    pub fn new(
        reducers: Vec<Reducer>,
        input: InputPort,
        output: OutputPort,
        policy: crosscut::policies::RuntimePolicy,
    ) -> Self {
        Worker {
            reducers,
            input,
            output,
            policy,
            ops_count: Default::default(),
        }
    }

    fn reduce_block<'b>(
        &mut self,
        block: &'b [u8],
        ctx: &model::BlockContext,
    ) -> Result<(), gasket::error::Error> {
        let block = MultiEraBlock::decode(block)
            .map_err(crate::Error::cbor)
            .apply_policy(&self.policy)
            .or_panic()?;

        let block = match block {
            Some(x) => x,
            None => return Ok(()),
        };

        self.output.send(gasket::messaging::Message::from(
            model::CRDTCommand::block_starting(&block),
        ))?;

        for reducer in self.reducers.iter_mut() {
            reducer.reduce_block(&block, ctx, &mut self.output)?;
            self.ops_count.inc(1);
        }

        self.output.send(gasket::messaging::Message::from(
            model::CRDTCommand::block_finished(&block),
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
        let msg = self.input.recv_or_idle()?;

        match msg.payload {
            model::EnrichedBlockPayload::RollForward(block, ctx) => {
                self.reduce_block(&block, &ctx)?
            }
            model::EnrichedBlockPayload::RollBack(point) => {
                log::warn!("rollback requested for {:?}", point);
            }
        }

        Ok(gasket::runtime::WorkOutcome::Partial)
    }
}
