use std::{str::FromStr, time::Duration};

use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};

use redis::{Commands, ToRedisArgs};
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

type InputPort = gasket::messaging::InputPort<model::CRDTCommand>;

impl ToRedisArgs for model::Value {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            model::Value::String(x) => x.write_redis_args(out),
            model::Value::Cbor(x) => x.write_redis_args(out),
        }
    }
}

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
    input: InputPort,
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    pub fn read_cursor(&mut self) -> Result<crosscut::Cursor, crate::Error> {
        let mut connection = redis::Client::open(self.config.connection_params.clone())
            .and_then(|x| x.get_connection())
            .map_err(crate::Error::storage)?;

        let raw: Option<String> = connection.get("_cursor").map_err(crate::Error::storage)?;

        let point = match raw {
            Some(x) => Some(crosscut::PointArg::from_str(&x)?),
            None => None,
        };

        Ok(point)
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        let worker = Worker {
            config: self.config.clone(),
            connection: None,
            input: self.input,
            ops_count: Default::default(),
        };

        pipeline.register_stage(
            spawn_stage(
                worker,
                gasket::runtime::Policy {
                    tick_timeout: Some(Duration::from_secs(600)),
                    ..Default::default()
                },
                Some("redis"),
            ),
        );
    }
}

pub struct Worker {
    config: Config,
    connection: Option<redis::Connection>,
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
            model::CRDTCommand::BlockStarting(_) => {
                // TODO: start transaction
            }
            model::CRDTCommand::GrowOnlySetAdd(key, value) => {
                self.connection
                    .as_mut()
                    .unwrap()
                    .sadd(key, value)
                    .or_restart()?;
            }
            model::CRDTCommand::TwoPhaseSetAdd(key, value) => {
                log::debug!("adding to 2-phase set [{}], value [{}]", key, value);

                self.connection
                    .as_mut()
                    .unwrap()
                    .sadd(key, value)
                    .or_restart()?;
            }
            model::CRDTCommand::TwoPhaseSetRemove(key, value) => {
                log::debug!("removing from 2-phase set [{}], value [{}]", key, value);

                self.connection
                    .as_mut()
                    .unwrap()
                    .sadd(format!("{}.ts", key), value)
                    .or_restart()?;
            }
            model::CRDTCommand::SetAdd(key, value) => {
                log::debug!("adding to set [{}], value [{}]", key, value);

                self.connection
                    .as_mut()
                    .unwrap()
                    .sadd(key, value)
                    .or_restart()?;
            }
            model::CRDTCommand::SetRemove(key, value) => {
                log::debug!("removing from set [{}], value [{}]", key, value);

                self.connection
                    .as_mut()
                    .unwrap()
                    .srem(key, value)
                    .or_restart()?;
            }
            model::CRDTCommand::LastWriteWins(key, value, ts) => {
                log::debug!("last write for [{}], slot [{}]", key, ts);

                self.connection
                    .as_mut()
                    .unwrap()
                    .zadd(key, value, ts)
                    .or_restart()?;
            }
            model::CRDTCommand::AnyWriteWins(key, value) => {
                log::debug!("overwrite [{}]", key);

                self.connection
                    .as_mut()
                    .unwrap()
                    .set(key, value)
                    .or_restart()?;
            }
            model::CRDTCommand::PNCounter(key, value) => {
                log::debug!("increasing counter [{}], by [{}]", key, value);

                self.connection
                    .as_mut()
                    .unwrap()
                    .incr(key, value)
                    .or_restart()?;
            }
            model::CRDTCommand::BlockFinished(point) => {
                let cursor_str = crosscut::PointArg::from(point).to_string();

                self.connection
                    .as_mut()
                    .unwrap()
                    .set("_cursor", &cursor_str)
                    .or_restart()?;

                log::info!("new cursor saved to redis {}", &cursor_str)
            }
        };

        self.ops_count.inc(1);

        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        self.connection = redis::Client::open(self.config.connection_params.clone())
            .and_then(|c| c.get_connection())
            .or_retry()?
            .into();

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        Ok(())
    }
}
