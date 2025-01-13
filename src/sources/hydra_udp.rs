use gasket::framework::*;
use serde::Deserialize;
use serde_json::{Value};
use tracing::{debug};

use pallas_primitives::conway::{MintedBlock};
use pallas::network::miniprotocols::Point;

use std::net::UdpSocket;

use crate::framework::errors::Error;
use crate::framework::*;

use crate::sources::hydra::{Event, StateChanged, make_conway_block};

use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Stage)]
#[stage(
    name = "source-hydra-udp",
    unit = "Event"
    worker = "Worker"
)]
pub struct Stage {
    config: Config,

    pub output: SourceOutputPort,

    #[metric]
    ops_count: gasket::metrics::Counter,

    #[metric]
    chain_tip: gasket::metrics::Gauge,
}

pub struct Worker {
    socket: UdpSocket,
    mempool: HashMap<String, String>,
}

impl Worker {
    async fn process_next(
        &mut self,
        stage: &mut Stage,
        next: &Event,
    ) -> Result<(), WorkerError> {
        match &next.state_changed {
            StateChanged::TransactionReceived(transaction_received) => {
                let cbor_hex = &transaction_received.tx.cbor_hex;
                self.mempool.insert(transaction_received.tx.tx_id.clone(), cbor_hex.clone());
                Ok(())
            }
            StateChanged::SnapshotConfirmed(snapshot_confirmed) => {
                let time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(n) => n.as_secs(),
                    Err(_) => panic!("SystemTime before UNIX EPOCH!"),
                };
                let mut block_transactions = vec![];
                for hash in &snapshot_confirmed.confirmed_transactions {
                    let tx_in_block = self.mempool.remove(hash).ok_or(WorkerError::Restart)?;
                    block_transactions.push(tx_in_block);
                }
                let block = make_conway_block(block_transactions, time).or_panic()?;
                let block_minted: MintedBlock = minicbor::decode(&block.cbor).or_panic()?;
                let wrapped_block = (7u16, block_minted);
                let block_cbor = minicbor::to_vec(wrapped_block).or_panic()?;
                let evt = ChainEvent::apply(
                    Point::Specific(block.slot, block.hash.to_vec()),
                    Record::RawBlockPayload(block_cbor.to_vec()),
                );
                stage.output.send(evt).await.or_panic()?;
                stage.chain_tip.set(time as i64);
                Ok(())
            }
            StateChanged::Unimplemented => {
                // Do nothing
                Ok(())
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl gasket::framework::Worker<Stage> for Worker {
    async fn bootstrap(stage: &Stage) -> Result<Self, WorkerError> {
        debug!("connecting");
        let socket = UdpSocket::bind(&stage.config.endpoint).unwrap();

        let worker = Self { socket, mempool: HashMap::new() };

        Ok(worker)
    }

    async fn schedule(
        &mut self,
        _stage: &mut Stage,
    ) -> Result<WorkSchedule<Event>, WorkerError> {
        let mut buf = [0; 1048576];
        let n = self.socket.recv(&mut buf).unwrap();
        let msg = &buf[..n];
        debug!("received udp: {}", std::str::from_utf8(&buf).unwrap());
        let v: Value = serde_json::from_slice(&msg).or_restart()?;
        let event = Event::try_from(v).or_restart()?;
        Ok(WorkSchedule::Unit(event))
    }

    async fn execute(
        &mut self,
        unit: &Event,
        stage: &mut Stage,
    ) -> Result<(), WorkerError> {
        self.process_next(stage, unit).await
    }

    async fn teardown(&mut self) -> Result<(), WorkerError> {
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    endpoint: String,
}

impl Config {
    pub fn bootstrapper(self, _ctx: &Context) -> Result<Stage, Error> {
        let stage = Stage {
            config: self,
            output: Default::default(),
            ops_count: Default::default(),
            chain_tip: Default::default(),
        };

        Ok(stage)
    }
}
