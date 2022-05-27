use std::str::FromStr;

use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};
use redis::Commands;
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

type FunnelPort = gasket::messaging::FunnelPort<model::CRDTCommand>;

#[derive(Deserialize)]
pub struct Config {
    pub connection_params: String,
}

pub struct Worker {
    config: Config,
    connection: Option<redis::Connection>,
    input: FunnelPort,
}

impl Worker {
    fn redis_connect(&self) -> Result<redis::Connection, redis::RedisError> {
        let client = redis::Client::open(self.config.connection_params.clone())?;
        client.get_connection()
    }
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
            model::CRDTCommand::BlockFinished(point) => self
                .connection
                .as_mut()
                .unwrap()
                .set("_cursor", point.to_string())
                .or_work_err()?,
        };

        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let connection = self.redis_connect().or_work_err()?;

        self.connection = Some(connection);

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        Ok(())
    }
}

impl super::Pluggable for Worker {
    fn borrow_input_port(&mut self) -> &'_ mut FunnelPort {
        &mut self.input
    }

    fn read_cursor(&self) -> Result<crosscut::Cursor, crate::Error> {
        let mut connection = self.redis_connect().map_err(crate::Error::storage)?;

        let raw: Option<String> = connection.get("_cursor").map_err(crate::Error::storage)?;

        let point = match raw {
            Some(x) => Some(crosscut::PointArg::from_str(&x)?),
            None => None,
        };

        Ok(point)
    }

    fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        pipeline.register_stage("redis", spawn_stage(self, Default::default()));
    }
}

impl Config {
    pub fn plugin(
        self,
        _chain: &crosscut::ChainWellKnownInfo,
        _intersect: &crosscut::IntersectConfig,
    ) -> super::Plugin {
        let worker = Worker {
            config: self,
            connection: None,
            input: Default::default(),
        };

        super::Plugin::Redis(worker)
    }
}
