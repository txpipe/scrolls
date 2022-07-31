use std::time::Duration;

use gasket::runtime::{spawn_stage, WorkOutcome};

use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

type InputPort = gasket::messaging::InputPort<model::CRDTCommand>;

#[derive(Deserialize, Clone)]
pub struct Config {}

impl Config {
    pub fn boostrapper(self) -> Bootstrapper {
        Bootstrapper {
            input: Default::default(),
        }
    }
}

pub struct Bootstrapper {
    input: InputPort,
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    pub fn read_cursor(&mut self) -> Result<crosscut::Cursor, crate::Error> {
        Ok(None)
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        let worker = Worker {
            input: self.input,
            ops_count: Default::default(),
        };

        pipeline.register_stage(
            "skip",
            spawn_stage(
                worker,
                gasket::runtime::Policy {
                    tick_timeout: Some(Duration::from_secs(5)),
                    ..Default::default()
                },
            ),
        );
    }
}

pub struct Worker {
    ops_count: gasket::metrics::Counter,
    input: InputPort,
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("storage_ops", &self.ops_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv_or_idle()?;

        match msg.payload {
            model::CRDTCommand::BlockStarting(point) => {
                log::debug!("block started {:?}", point);
            }
            model::CRDTCommand::GrowOnlySetAdd(key, value) => {
                log::debug!("adding to grow-only set [{}], value [{}]", key, value);
            }
            model::CRDTCommand::TwoPhaseSetAdd(key, value) => {
                log::debug!("adding to 2-phase set [{}], value [{}]", key, value);
            }
            model::CRDTCommand::TwoPhaseSetRemove(key, value) => {
                log::debug!("removing from 2-phase set [{}], value [{}]", key, value);
            }
            model::CRDTCommand::SetAdd(key, value) => {
                log::debug!("adding to set [{}], value [{}]", key, value);
            }
            model::CRDTCommand::SetRemove(key, value) => {
                log::debug!("removing from set [{}], value [{}]", key, value);
            }
            model::CRDTCommand::LastWriteWins(key, _, ts) => {
                log::debug!("last write for [{}], slot [{}]", key, ts);
            }
            model::CRDTCommand::AnyWriteWins(key, _) => {
                log::debug!("overwrite [{}]", key);
            }
            model::CRDTCommand::PNCounter(key, value) => {
                log::debug!("increasing counter [{}], by [{}]", key, value);
            }
            model::CRDTCommand::BlockFinished(point) => {
                log::debug!("block finished {:?}", point);
            }
        };

        self.ops_count.inc(1);

        Ok(WorkOutcome::Partial)
    }
}
