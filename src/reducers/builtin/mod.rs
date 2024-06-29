use gasket::framework::*;
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::framework::model::CRDTCommand;
use crate::framework::*;

mod full_utxos_by_address;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ReducerConfig {
    FullUtxosByAddress(full_utxos_by_address::Config),
}

impl ReducerConfig {
    pub fn into_reducer(self) -> Box<dyn ReducerTrait> {
        match self {
            ReducerConfig::FullUtxosByAddress(x) => x.plugin(),
        }
    }
}

#[derive(Deserialize)]
pub struct Config {
    reducers: Vec<ReducerConfig>,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            reducers: self
                .reducers
                .into_iter()
                .map(|x| x.into_reducer())
                .collect(),
            ..Default::default()
        };

        Ok(stage)
    }
}

#[derive(Default, Stage)]
#[stage(name = "reducer-builtin", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    reducers: Vec<Box<dyn ReducerTrait>>,

    pub input: ReducerInputPort,
    pub output: ReducerOutputPort,

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
            .or_panic()?;

            let mut commands: Vec<CRDTCommand> = Vec::new();

            for x in stage.reducers.iter_mut() {
                commands.append(&mut x.reduce_block(&block, ctx).await.or_retry()?)
            }

            Ok(commands)
        },
        _ => todo!(),
    }?;

    Some(ChainEvent::apply(unit.point().clone(), Record::CRDTCommand(commands)))
});

#[async_trait::async_trait]
pub trait ReducerTrait: Send + Sync {
    async fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
    ) -> Result<Vec<CRDTCommand>, Error>;
}

trait ReducerConfigTrait {
    fn plugin(self) -> Box<dyn ReducerTrait>;
}
