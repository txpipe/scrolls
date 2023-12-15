use gasket::framework::*;
use gasket::{
    messaging::{RecvPort, SendPort},
    runtime::Tether,
};
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::framework::model::CRDTCommand;
use crate::framework::*;

#[derive(Deserialize)]

pub struct Config {
    // TODO: specify javascript file
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            ..Default::default()
        };

        Ok(stage)
    }
}

#[derive(Default, Stage)]
#[stage(name = "reducer", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: ReducerInputPort,
    pub output: ReducerOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

impl StageBootstrapper for Stage {
    fn connect_input(&mut self, adapter: InputAdapter) {
        self.input.connect(adapter)
    }

    fn connect_output(&mut self, adapter: OutputAdapter) {
        self.output.connect(adapter)
    }

    fn spawn(self, policy: gasket::runtime::Policy) -> Tether {
        gasket::runtime::spawn_stage(self, policy)
    }
}

#[derive(Default)]
pub struct Worker;

impl From<&Stage> for Worker {
    fn from(_: &Stage) -> Self {
        Self
    }
}

gasket::impl_splitter!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let record = unit.record();
    if record.is_none() {
        return Ok(());
    }

    let record = record.unwrap();

    let commands = match record {
        Record::EnrichedBlockPayload(block, ctx) => {
            let block = MultiEraBlock::decode(block)
            .map_err(Error::cbor)
            // .apply_policy(&self.policy)
            .or_panic()?;

            let commands  = vec![];

            // TODO: call deno runtime

            Ok(commands)
        },
        _ => todo!(),
    }?;

    Some(ChainEvent::apply(unit.point().clone(), Record::CRDTCommand(commands)))
});
