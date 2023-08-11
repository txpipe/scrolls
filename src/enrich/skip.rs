use std::time::Duration;

use gasket::runtime::{spawn_stage, WorkOutcome};

use crate::{
    bootstrap,
    model::{self, BlockContext},
};

type InputPort = gasket::messaging::TwoPhaseInputPort<model::RawBlockPayload>;
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

        pipeline.register_stage(spawn_stage(
            worker,
            gasket::runtime::Policy {
                tick_timeout: Some(Duration::from_secs(600)),
                ..Default::default()
            },
            Some("enrich-skip"),
        ));
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
        let msg = self.input.recv_or_idle()?;

        match msg.payload {
            model::RawBlockPayload::RollForward(cbor) => {
                self.output.send(model::EnrichedBlockPayload::roll_forward(
                    cbor,
                    BlockContext::default(),
                ))?;
            }
            model::RawBlockPayload::RollBack(cbor) => {
                self.output
                    .send(model::EnrichedBlockPayload::roll_back(cbor, BlockContext::default()))?;
            }
        };

        self.input.commit();
        Ok(WorkOutcome::Partial)
    }
}
