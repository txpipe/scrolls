use std::time::Duration;

use futures::stream::StreamExt;

use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};

use mongodb::{
    bson::{doc, spec::BinarySubtype, Binary, Bson, Decimal128, Document},
    options::ClientOptions,
    Client, Database,
};
use serde::Deserialize;

use crate::prelude::*;
use crate::{bootstrap, crosscut, model, Error};

type InputPort = gasket::messaging::TwoPhaseInputPort<model::CRDTCommand>;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub connection_url: String,
    pub database: String,
    pub worker_threads: Option<usize>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Config {
    pub fn bootstrapper(
        self,
        _chain: &crosscut::ChainWellKnownInfo,
        _intersect: &crosscut::IntersectConfig,
        policy: &crosscut::policies::RuntimePolicy,
    ) -> Bootstrapper {
        Bootstrapper {
            config: self,
            policy: policy.clone(),
            input: Default::default(),
        }
    }
}

pub struct Bootstrapper {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    input: InputPort,
}

impl Bootstrapper {
    pub fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    pub fn build_cursor(&self) -> Cursor {
        Cursor {}
    }

    pub fn spawn_stages(self, pipeline: &mut bootstrap::Pipeline) {
        let threads = self.config.worker_threads.unwrap_or(3);

        let worker = Worker {
            config: self.config,
            policy: self.policy,
            database: None,
            runtime: tokio::runtime::Builder::new_multi_thread()
                .worker_threads(threads)
                .enable_io()
                .enable_time()
                .build()
                .expect("couldn't setup tokio async runtime"),
            input: self.input,
            ops_count: Default::default(),
        };

        pipeline.register_stage(spawn_stage(
            worker,
            gasket::runtime::Policy {
                tick_timeout: Some(Duration::from_secs(600)),
                bootstrap_retry: gasket::retries::Policy {
                    max_retries: 20,
                    backoff_unit: Duration::from_secs(1),
                    backoff_factor: 2,
                    max_backoff: Duration::from_secs(60),
                },
                ..Default::default()
            },
            Some("mongodb"),
        ));
    }
}

pub struct Cursor {}

impl Cursor {
    pub fn last_point(&mut self) -> Result<Option<crosscut::PointArg>, crate::Error> {
        Ok(None)
    }
}

pub struct Worker {
    config: Config,
    database: Option<Database>,
    runtime: tokio::runtime::Runtime,
    policy: crosscut::policies::RuntimePolicy,
    ops_count: gasket::metrics::Counter,
    input: InputPort,
}

impl Worker {
    async fn connect(&self) -> Result<Database, gasket::error::Error> {
        // Parse a connection string into an options struct.
        let options = ClientOptions::parse(&self.config.connection_url)
            .await
            .map_err(|err| Error::config(err.to_string()))
            .or_panic()?;

        let client = Client::with_options(options).or_restart()?;
        let database = client.database(&self.config.database);

        Ok(database)
    }
}

const BATCH_SIZE: usize = 40;

#[derive(Default)]
struct Batch {
    block_end: Option<model::CRDTCommand>,
    items: Vec<model::CRDTCommand>,
}

fn recv_batch(input: &mut InputPort) -> Result<Batch, gasket::error::Error> {
    let mut batch = Batch::default();

    loop {
        match input.recv_or_idle() {
            Ok(x) => match x.payload {
                model::CRDTCommand::BlockStarting(_) => (),
                model::CRDTCommand::BlockFinished(_) => {
                    batch.block_end = Some(x.payload);
                    return Ok(batch);
                }
                _ => {
                    batch.items.push(x.payload);
                }
            },
            Err(gasket::error::Error::RecvIdle) => return Ok(batch),
            Err(err) => return Err(err),
        };

        if batch.items.len() >= BATCH_SIZE {
            return Ok(batch);
        }
    }
}

fn value_to_bson(value: model::Value) -> Bson {
    match value {
        model::Value::String(x) => x.into(),
        model::Value::BigInt(x) => Bson::Decimal128(Decimal128::from_bytes(x.to_be_bytes())),
        model::Value::Cbor(x) => Bson::Binary(Binary {
            bytes: x,
            subtype: BinarySubtype::UserDefined(1),
        }),
        model::Value::Json(x) => Bson::try_from(x).unwrap(),
    }
}

// TODO: collection name should be defined by the prefix of the key of the
// command. The problem is that we're concatenating the string upstream, so we
// don't have access to the clean prefix value. We could parse it, but it's a
// very ugly solution. A solution I like better is to keep the key prefix as an
// independent value in the command enum and let the storage stage decide how to
// persist it. The later solution requires several changes that we'll be treated
// as a different PR. In the meantime, we'll keep working in this MongoDb sink
// sending everything to a single, hardcoded collection.
const BAD_HARDCODED_COLLECTION: &str = &"catchall";

async fn apply_command(
    cmd: model::CRDTCommand,
    db: &Database,
) -> Result<(), mongodb::error::Error> {
    match cmd {
        model::CRDTCommand::BlockStarting(_) => Ok(()),
        model::CRDTCommand::AnyWriteWins(key, value) => {
            db.collection::<Document>(BAD_HARDCODED_COLLECTION)
                .update_one(
                    doc! { "_id": &key },
                    doc! { "_id": &key, "value": value_to_bson(value) },
                    None,
                )
                .await?;

            Ok(())
        }
        model::CRDTCommand::BlockFinished(_) => {
            log::warn!("MongoDb storage doesn't support cursors ATM");
            Ok(())
        }
        _ => todo!(),
    }
}

async fn apply_batch(
    batch: Batch,
    db: &Database,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<(), gasket::error::Error> {
    let mut stream = futures::stream::iter(batch.items)
        .map(|cmd| apply_command(cmd, db))
        .buffer_unordered(10);

    while let Some(op) = stream.next().await {
        op.map_err(|e| Error::StorageError(e.to_string()))
            .apply_policy(policy)
            .or_retry()?;
    }

    // we process the block end after the rest of the commands to ensure that no
    // other change from the block remains pending in the async queue
    if let Some(block_end) = batch.block_end {
        apply_command(block_end, db)
            .await
            .map_err(|e| Error::StorageError(e.to_string()))
            .apply_policy(policy)
            .or_panic()?;
    }

    Ok(())
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("storage_ops", &self.ops_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let batch = recv_batch(&mut self.input)?;
        let count = batch.items.len();
        let db = self.database.as_ref().unwrap();

        self.runtime
            .block_on(async { apply_batch(batch, db, &self.policy).await })?;

        self.ops_count.inc(count as u64);
        self.input.commit();

        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let db = self.runtime.block_on(self.connect())?;
        self.database = Some(db);

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        Ok(())
    }
}
