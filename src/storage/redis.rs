use gasket::framework::*;
use r2d2_redis::{
    r2d2::{self, Pool},
    redis::{self, Commands, ToRedisArgs},
    RedisConnectionManager,
};
use serde::Deserialize;
use std::ops::DerefMut;
use tracing::debug;

use crate::framework::*;

pub struct Worker {
    pool: Pool<RedisConnectionManager>,
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        let manager =
            RedisConnectionManager::new(stage.config.url.clone()).or_panic()?;
        let pool = r2d2::Pool::builder().build(manager).or_panic()?;

        Ok(Self { pool })
    }

    async fn schedule(
        &mut self,
        stage: &mut Stage,
    ) -> Result<WorkSchedule<ChainEvent>, WorkerError> {
        let msg = stage.input.recv().await.or_panic()?;
        Ok(WorkSchedule::Unit(msg.payload))
    }

    async fn execute(&mut self, unit: &ChainEvent, stage: &mut Stage) -> Result<(), WorkerError> {
        let point = unit.point().clone();
        let record = unit.record().cloned();

        if record.is_none() {
            return Ok(());
        }

        let record = record.unwrap();

        match record {
            Record::CRDTCommand(commands) => {
                let mut conn = self.pool.get().or_restart()?;

                redis::cmd("MULTI").query(conn.deref_mut()).or_retry()?;

                // TODO: add to parallel persist
                for command in commands {
                    match command {
                        model::CRDTCommand::GrowOnlySetAdd(key, value) => {
                            conn.deref_mut().sadd(key, value).or_restart()?;
                        }
                        model::CRDTCommand::TwoPhaseSetAdd(key, value) => {
                            debug!(key, value, "adding to 2-phase");

                            conn.deref_mut().sadd(key, value).or_restart()?;
                        }
                        model::CRDTCommand::TwoPhaseSetRemove(key, value) => {
                            debug!(key, value, "removing from 2-phase");

                            conn.deref_mut()
                                .sadd(format!("{}.ts", key), value)
                                .or_restart()?;
                        }
                        model::CRDTCommand::SetAdd(key, value) => {
                            debug!(key, value, "adding");

                            conn.deref_mut().sadd(key, value).or_restart()?;
                        }
                        model::CRDTCommand::SetRemove(key, value) => {
                            debug!(key, value, "removing");

                            conn.deref_mut().srem(key, value).or_restart()?;
                        }
                        model::CRDTCommand::LastWriteWins(key, value, slot) => {
                            debug!(key, slot, "last write");

                            conn.deref_mut().zadd(key, value, slot).or_restart()?;
                        }
                        model::CRDTCommand::SortedSetAdd(key, value, delta) => {
                            debug!(key, value, delta, "sorted set add");

                            conn.deref_mut().zincr(key, value, delta).or_restart()?;
                        }
                        model::CRDTCommand::SortedSetRemove(key, value, delta) => {
                            debug!(key, value, delta, "sorted set remove");

                            conn.deref_mut().zincr(&key, value, delta).or_restart()?;

                            // removal of dangling scores  (aka garage collection)
                            conn.deref_mut().zrembyscore(&key, 0, 0).or_restart()?;
                        }
                        model::CRDTCommand::AnyWriteWins(key, value) => {
                            debug!(key, "overwrite");

                            conn.deref_mut().set(key, value).or_restart()?;
                        }
                        model::CRDTCommand::PNCounter(key, value) => {
                            debug!(key, value, "increasing counter");

                            conn.deref_mut().incr(key, value).or_restart()?;
                        }
                        model::CRDTCommand::HashSetValue(key, member, value) => {
                            debug!(key, member, "setting hash");

                            conn.deref_mut().hset(key, member, value).or_restart()?;
                        }
                        model::CRDTCommand::HashCounter(key, member, delta) => {
                            debug!(key, member, delta, "increasing hash");

                            conn.deref_mut().hincr(key, member, delta).or_restart()?;
                        }
                        model::CRDTCommand::HashUnsetKey(key, member) => {
                            debug!(key, member, "deleting hash");

                            conn.deref_mut().hdel(member, key).or_restart()?;
                        }
                    }
                }

                // TODO: check if its necessary to set cursor
                redis::cmd("EXEC").query(conn.deref_mut()).or_retry()?;
            }
            _ => todo!(),
        }

        stage.ops_count.inc(1);
        stage.latest_block.set(point.slot_or_default() as i64);
        stage.cursor.add_breadcrumb(point.clone());

        Ok(())
    }
}

#[derive(Stage)]
#[stage(name = "redis", unit = "ChainEvent", worker = "Worker")]
pub struct Stage {
    config: Config,
    cursor: Cursor,

    pub input: StorageInputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    latest_block: gasket::metrics::Gauge,
}

#[derive(Default, Deserialize)]
pub struct Config {
    pub url: String,
}

impl Config {
    pub fn bootstrapper(self, ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            cursor: ctx.cursor.clone(),
            ops_count: Default::default(),
            latest_block: Default::default(),
            input: Default::default(),
        };

        Ok(stage)
    }
}

impl ToRedisArgs for model::Value {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        match self {
            model::Value::String(x) => x.write_redis_args(out),
            model::Value::BigInt(x) => x.to_string().write_redis_args(out),
            model::Value::Cbor(x) => x.write_redis_args(out),
            model::Value::Json(x) => todo!("{}", x),
        }
    }
}
