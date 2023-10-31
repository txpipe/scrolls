use gasket::framework::*;
use gasket::{
    messaging::{RecvPort, SendPort},
    runtime::Tether,
};
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::framework::model::CRDTCommand;
use crate::framework::*;

mod full_utxos_by_address;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Config {
    FullUtxosByAddress(full_utxos_by_address::Config),
}
impl Config {
    pub fn plugin(self) -> Box<dyn ReducerTrait> {
        match self {
            Config::FullUtxosByAddress(c) => c.plugin(),
        }
    }
}

pub trait ConfigTrait {
    fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error>;
}
impl ConfigTrait for Vec<Config> {
    fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let reducers: Vec<Box<dyn ReducerTrait>> =
            self.into_iter().map(|c: Config| c.plugin()).collect();

        let stage = Stage {
            reducers,
            ..Default::default()
        };

        Ok(stage)
    }
}

#[derive(Default, Stage)]
#[stage(name = "reducer", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    reducers: Vec<Box<dyn ReducerTrait>>,

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
