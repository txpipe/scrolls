use gasket::framework::*;
use serde::Deserialize;

use crate::framework::{
    model::{BlockContext, EnrichedBlockPayload, RawBlockPayload},
    *,
};

#[derive(Default, Stage)]
#[stage(name = "enrich-skip", unit = "RawBlockPayload", worker = "Worker")]
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

gasket::impl_mapper!(|_worker: Worker, stage: Stage, unit: RawBlockPayload| => {
    let evt = match unit {
        RawBlockPayload::RollForward(cbor) => EnrichedBlockPayload::roll_forward(cbor.clone(), BlockContext::default()),
        RawBlockPayload::RollBack(point) => EnrichedBlockPayload::roll_back(point.clone())
    };

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
