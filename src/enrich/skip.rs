use gasket::runtime::{spawn_stage, WorkOutcome};

use crate::{
    bootstrap,
    model::{self, BlockContext},
};

type InputPort = gasket::messaging::InputPort<model::RawBlockPayload>;
type OutputPort = gasket::messaging::OutputPort<model::EnrichedBlockPayload>;

pub struct Bootstrapper {
    input: InputPort,
    output: OutputPort,
}

impl Default for Bootstrapper {
    fn default() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
        }
    }
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    pub fn borrow_output_port(&mut self) -> &'_ mut OutputPort {
        &mut self.output
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        let worker = Worker {
            input: self.input,
            output: self.output,
        };

        pipeline.register_stage("enrich-skip", spawn_stage(worker, Default::default()));
    }
}

pub struct Worker {
    input: InputPort,
    output: OutputPort,
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new().build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv()?;

        match msg.payload {
            model::RawBlockPayload::RollForward(cbor) => {
                self.output.send(model::EnrichedBlockPayload::roll_forward(
                    cbor,
                    BlockContext::default(),
                ))?;
            }
            model::RawBlockPayload::RollBack(x) => {
                self.output
                    .send(model::EnrichedBlockPayload::roll_back(x))?;
            }
        };

        Ok(WorkOutcome::Partial)
    }
}
