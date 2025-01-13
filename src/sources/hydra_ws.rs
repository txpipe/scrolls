use gasket::framework::*;
use serde::Deserialize;
use serde_json::{Value};
use tracing::{debug};

use pallas_primitives::conway::{MintedBlock};
use pallas::network::miniprotocols::Point;

use crate::framework::errors::Error;
use crate::framework::*;

use crate::sources::hydra::{HydraWsMessage, make_conway_block};

use std::collections::HashMap;
use std::time::SystemTime;

use tokio_tungstenite::connect_async;
use futures::stream::SplitStream;
use futures::StreamExt;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio::net::TcpStream;
use tungstenite::client::IntoClientRequest;

#[derive(Stage)]
#[stage(
    name = "source-hydra-ws",
    unit = "HydraWsMessage"
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

type HydraStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub struct Worker {
    socket: HydraStream,
    mempool: HashMap<String, String>,
}

impl Worker {
    async fn process_next(
        &mut self,
        stage: &mut Stage,
        next: &HydraWsMessage,
    ) -> Result<(), WorkerError> {
        match &next {
            HydraWsMessage::TransactionReceived(transaction_received) => {
                let cbor_hex = &transaction_received.tx.cbor_hex;
                self.mempool.insert(transaction_received.tx.tx_id.clone(), cbor_hex.clone());
                Ok(())
            }
            HydraWsMessage::SnapshotConfirmed(snapshot_confirmed) => {
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
            HydraWsMessage::TxValid((txid, txbody)) => {
                self.mempool.insert(txid.clone(), txbody.clone());
                Ok(())
            }
            HydraWsMessage::Unimplemented(_) => {
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
        let request = stage.config.endpoint.clone().into_client_request().unwrap();
        let (stream, _) = connect_async(request).await.unwrap();
        let (_hydra_node_write, hydra_node_read) = stream.split();

        let worker = Self { socket: hydra_node_read, mempool: HashMap::new() };

        Ok(worker)
    }

    async fn schedule(
        &mut self,
        _stage: &mut Stage,
    ) -> Result<WorkSchedule<HydraWsMessage>, WorkerError> {
        loop {
            let msg = self.socket.next().await.unwrap().unwrap();
            if msg.is_binary() || msg.is_text() {
                let bytes = msg.into_data();
                let v: Value = serde_json::from_slice(&bytes).or_restart()?;
                let hydra_ws_message = HydraWsMessage::try_from(v).or_restart()?;
                return Ok(WorkSchedule::Unit(hydra_ws_message))
            }
        }
    }

    async fn execute(
        &mut self,
        unit: &HydraWsMessage,
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
