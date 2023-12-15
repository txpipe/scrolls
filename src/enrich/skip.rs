use gasket::framework::*;
use serde::Deserialize;

use crate::framework::{model::BlockContext, *};

#[derive(Default, Stage)]
#[stage(name = "enrich-skip", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    pub input: EnrichInputPort,
    pub output: EnrichOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,
}

#[derive(Default)]
pub struct Worker;

impl From<&Stage> for Worker {
    fn from(_: &Stage) -> Self {
        Self
    }
}

gasket::impl_mapper!(|_worker: Worker, stage: Stage, unit: ChainEvent| => {
    let evt = match unit {
        ChainEvent::Apply(point, record) => {
            match record {
                Record::RawBlockPayload(cbor) => Ok(ChainEvent::apply(point.clone(), Record::EnrichedBlockPayload(cbor.clone(), BlockContext::default()))),
                _ => Err(WorkerError::Panic)
            }
        },
        ChainEvent::Reset(point) => Ok(ChainEvent::reset(point.clone())),
    }?;

    stage.ops_count.inc(1);

    evt
});

#[derive(Default, Deserialize)]
pub struct Config {}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        Ok(Stage::default())
    }
}
