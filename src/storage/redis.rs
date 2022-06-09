use std::str::FromStr;

use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};

use redis::{Commands, Connection};
use serde::Deserialize;

use crate::{
    bootstrap, crosscut,
    model::{self, StateData},
};

type FunnelPort = gasket::messaging::FunnelPort<model::CRDTCommand>;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub connection_params: String,
}

impl Config {
    pub fn boostrapper(
        self,
        _chain: &crosscut::ChainWellKnownInfo,
        _intersect: &crosscut::IntersectConfig,
    ) -> Bootstrapper {
        Bootstrapper {
            config: self,
            input: Default::default(),
        }
    }
}

pub struct Bootstrapper {
    config: Config,
    input: FunnelPort,
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut FunnelPort {
        &mut self.input
    }

    pub fn build_read_plugin(&self) -> ReadPlugin {
        ReadPlugin {
            config: self.config.clone(),
            connection: None,
        }
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        let worker = Worker {
            config: self.config.clone(),
            connection: None,
            input: self.input,
        };

        pipeline.register_stage("redis", spawn_stage(worker, Default::default()));
    }
}

pub struct Worker {
    config: Config,
    connection: Option<redis::Connection>,
    input: FunnelPort,
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new().build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv()?;

        match msg.payload {
            model::CRDTCommand::BlockStarting(_) => {
                // TODO: start transaction
            }
            model::CRDTCommand::GrowOnlySetAdd(key, value) => {
                self.connection
                    .as_mut()
                    .unwrap()
                    .sadd(key, value)
                    .or_work_err()?;
            }
            model::CRDTCommand::TwoPhaseSetAdd(key, value) => {
                self.connection
                    .as_mut()
                    .unwrap()
                    .sadd(key, value)
                    .or_work_err()?;
            }
            model::CRDTCommand::TwoPhaseSetRemove(key, value) => {
                self.connection
                    .as_mut()
                    .unwrap()
                    .sadd(format!("{}.ts", key), value)
                    .or_work_err()?;
            }
            model::CRDTCommand::LastWriteWins(key, value, timestamp) => {
                self.connection
                    .as_mut()
                    .unwrap()
                    .zadd(key, value, timestamp)
                    .or_work_err()?;
            }
            model::CRDTCommand::AnyWriteWins(key, value) => {
                self.connection
                    .as_mut()
                    .unwrap()
                    .set(key, value)
                    .or_work_err()?;
            }
            model::CRDTCommand::PNCounter(key, value) => {
                self.connection
                    .as_mut()
                    .unwrap()
                    .incr(key, value)
                    .or_work_err()?;
            }
            model::CRDTCommand::BlockFinished(point) => {
                let cursor_str = crosscut::PointArg::from(point).to_string();

                self.connection
                    .as_mut()
                    .unwrap()
                    .set("_cursor", &cursor_str)
                    .or_work_err()?;

                log::info!("new cursor saved to redis {}", &cursor_str)
            }
        };

        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let client = redis::Client::open(self.config.connection_params.clone()).or_work_err()?;
        let connection = client.get_connection().or_work_err()?;

        self.connection = Some(connection);

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        Ok(())
    }
}

pub struct ReadPlugin {
    config: Config,
    connection: Option<Connection>,
}

impl ReadPlugin {
    pub fn bootstrap(&mut self) -> Result<(), crate::Error> {
        let connection = redis::Client::open(self.config.connection_params.clone())
            .and_then(|c| c.get_connection())
            .map_err(crate::Error::storage)?;

        self.connection = Some(connection);

        Ok(())
    }

    pub fn read_state(
        &mut self,
        query: model::StateQuery,
    ) -> Result<model::StateData, crate::Error> {
        let value = match query {
            model::StateQuery::KeyValue(key) => self
                .connection
                .as_mut()
                .unwrap()
                .get(key)
                .map(|v| StateData::KeyValue(v))
                .map_err(crate::Error::storage)?,
            model::StateQuery::LatestKeyValue(key) => self
                .connection
                .as_mut()
                .unwrap()
                .zrevrange(key, 0, 1)
                .map(|v| StateData::KeyValue(v))
                .map_err(crate::Error::storage)?,
            model::StateQuery::SetMembers(key) => self
                .connection
                .as_mut()
                .unwrap()
                .get(key)
                .map(|v| StateData::SetMembers(v))
                .map_err(crate::Error::storage)?,
        };

        Ok(value.into())
    }

    pub fn read_cursor(&mut self) -> Result<crosscut::Cursor, crate::Error> {
        let raw: Option<String> = self
            .connection
            .as_mut()
            .unwrap()
            .get("_cursor")
            .map_err(crate::Error::storage)?;

        let point = match raw {
            Some(x) => Some(crosscut::PointArg::from_str(&x)?),
            None => None,
        };

        Ok(point)
    }
}
