use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};
use redis::Commands;
use serde::Deserialize;

use crate::{bootstrap, model};

type InputPort = gasket::messaging::InputPort<model::CRDTCommand>;

#[derive(Deserialize)]
pub struct Config {
    pub connection_params: String,
}

pub struct Worker {
    config: Config,
    client: Option<redis::Client>,
    connection: Option<redis::Connection>,
    input: InputPort,
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new().build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv()?;

        match msg.payload {
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
        };

        Ok(WorkOutcome::Partial)
    }

    fn bootstrap(&mut self) -> Result<(), gasket::error::Error> {
        let client = redis::Client::open(self.config.connection_params.clone()).or_work_err()?;
        let connection = client.get_connection().or_work_err()?;

        self.client = Some(client);
        self.connection = Some(connection);

        Ok(())
    }

    fn teardown(&mut self) -> Result<(), gasket::error::Error> {
        Ok(())
    }
}

impl super::Pluggable for Worker {
    fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        pipeline.register_stage("redis", spawn_stage(self, Default::default()));
    }
}

impl From<Config> for Worker {
    fn from(other: Config) -> Self {
        Worker {
            config: other,
            client: None,
            connection: None,
            input: Default::default(),
        }
    }
}
