use std::time::Duration;

use elasticsearch::{http::response::Response, Elasticsearch};
use futures::stream::StreamExt;

use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};

use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

use crate::{
    bootstrap, crosscut,
    model::{self, CRDTCommand},
    prelude::AppliesPolicy,
    Error,
};

type InputPort = gasket::messaging::InputPort<model::CRDTCommand>;

impl Into<JsonValue> for model::Value {
    fn into(self) -> JsonValue {
        match self {
            model::Value::String(x) => json!({ "value": x }),
            model::Value::Cbor(x) => json!({ "cbor": hex::encode(x) }),
            model::Value::Json(x) => x,
            model::Value::BigInt(x) => json!({ "value": x }),
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub connection_url: String,
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
            client: None,
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
            Some("elastic"),
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
    client: Option<Elasticsearch>,
    runtime: tokio::runtime::Runtime,
    policy: crosscut::policies::RuntimePolicy,
    ops_count: gasket::metrics::Counter,
    input: InputPort,
}

const BATCH_SIZE: usize = 40;

#[derive(Default)]
struct Batch {
    block_end: Option<CRDTCommand>,
    items: Vec<CRDTCommand>,
}

fn recv_batch(input: &mut InputPort) -> Result<Batch, gasket::error::Error> {
    let mut batch = Batch::default();

    loop {
        match input.recv_or_idle() {
            Ok(x) => match x.payload {
                CRDTCommand::BlockStarting(_) => (),
                CRDTCommand::BlockFinished(_) => {
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

type ESResult = Result<Response, elasticsearch::Error>;

async fn apply_command(cmd: CRDTCommand, client: &Elasticsearch) -> Option<ESResult> {
    match cmd {
        CRDTCommand::BlockStarting(_) => None,
        CRDTCommand::AnyWriteWins(key, value) => client
            .index(elasticsearch::IndexParts::IndexId("scrolls", &key))
            .body::<JsonValue>(value.into())
            .send()
            .await
            .into(),
        CRDTCommand::BlockFinished(_) => {
            log::warn!("Elasticsearch storage doesn't support cursors ATM");
            None
        }
        _ => todo!(),
    }
}

async fn apply_batch(
    batch: Batch,
    client: &Elasticsearch,
    policy: &crosscut::policies::RuntimePolicy,
) -> Result<(), gasket::error::Error> {
    let mut stream = futures::stream::iter(batch.items)
        .map(|cmd| apply_command(cmd, client))
        .buffer_unordered(10);

    while let Some(op) = stream.next().await {
        if let Some(result) = op {
            // TODO: we panic because retrying a partial batch might yield weird results.
            // Once we have a two-phase commit mechanism in the input port, we can switch
            // back to retying instead of panicking.
            result
                .map(|x| x.error_for_status_code())
                .map_err(|e| Error::StorageError(e.to_string()))
                .apply_policy(policy)
                .or_panic()?;

            log::warn!("op executed on elastic");
        }
    }

    // we process the block end after the rest of the commands to ensure that no
    // other change from the block remains pending in the async queue
    if let Some(block_end) = batch.block_end {
        if let Some(result) = apply_command(block_end, client).await {
            result
                .and_then(|x| x.error_for_status_code())
                .map_err(|e| Error::StorageError(e.to_string()))
                .apply_policy(policy)
                .or_panic()?;
        }
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
        let client = self.client.as_ref().unwrap();

        self.runtime
            .block_on(async { apply_batch(batch, client, &self.policy).await })?;

        self.ops_count.inc(count as u64);
        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let url = elasticsearch::http::Url::parse(&self.config.connection_url)
            .map_err(|err| Error::ConfigError(err.to_string()))
            .or_panic()?;

        let auth = (&self.config.username, &self.config.password);

        let pool = elasticsearch::http::transport::SingleNodeConnectionPool::new(url);

        let transport = elasticsearch::http::transport::TransportBuilder::new(pool);

        let transport = if let (Some(username), Some(password)) = auth {
            transport.auth(elasticsearch::auth::Credentials::Basic(
                username.clone(),
                password.clone(),
            ))
        } else {
            transport
        };

        let transport = transport
            .cert_validation(elasticsearch::cert::CertificateValidation::None)
            .build()
            .or_retry()?;

        self.client = Elasticsearch::new(transport).into();

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        Ok(())
    }
}
