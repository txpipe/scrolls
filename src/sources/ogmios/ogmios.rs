use std::fmt;
use std::fmt::Display;

use pallas::network::miniprotocols::Point;

use tracing::{debug};
use anyhow::{Context, Result, anyhow};
use serde_json::{json, Value};
use tokio::net::TcpStream;
use futures::SinkExt;
use futures::StreamExt;
use futures::stream::{SplitSink, SplitStream};
use tungstenite::Message;
use tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub enum OgmiosResponse {
    RollForward(OgmiosRollForward),
    RollBackward(OgmiosRollBackward),
}

impl TryFrom<Value> for OgmiosResponse {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let next_block_response = value.as_object().context("Invalid NextBlock response")?;
        let result = next_block_response["result"].as_object().context("Invalid result")?;
        let direction = result["direction"].as_str().context("Invalid direction")?;
        if direction == "forward" {
            let rf = OgmiosRollForward::try_from(next_block_response["result"].clone())?;
            Ok(OgmiosResponse::RollForward(rf))
        } else if direction == "backward" {
            let rb = OgmiosRollBackward::try_from(next_block_response["result"].clone())?;
            Ok(OgmiosResponse::RollBackward(rb))
        } else {
            Err(anyhow!("direction: expected 'forward' or 'backward'"))
        }
    }
}

#[derive(Debug)]
pub struct OgmiosRollForward {
    pub slot: u64,
    pub transactions: Vec<Transaction>,
    pub tip: Tip,
}

#[derive(Debug)]
pub struct Transaction {
    pub cbor: String,
}

#[derive(Debug)]
pub struct Tip {
    pub height: u64,
    pub id: String,
    pub slot: u64,
}

impl TryFrom<Value> for OgmiosRollForward {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let block = value["block"].as_object().context("Invalid rollforward")?;
        let transactions = block["transactions"]
            .as_array()
            .context("Invalid transactions")?
            .iter()
            .map(|tx| {
                    Transaction::try_from(tx.clone())
                        .context("Invalid transaction")
            })
            .collect::<Result<Vec<Transaction>>>()?;
        let slot = block["slot"]
            .as_u64()
            .context("Invalid slot")?;
        let tip = Tip::try_from(value["tip"].clone()).context("Invalid tip")?;
        Ok(OgmiosRollForward{
            slot: slot,
            transactions: transactions,
            tip: tip,
        })
    }
}

impl TryFrom<Value> for Transaction {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let tx = value.as_object().context("Invalid transaction")?;
        let cbor = tx["cbor"].as_str().context("Invalid cbor")?;
        Ok(Transaction {
            cbor: cbor.to_string(),
        })
    }
}

impl TryFrom<Value> for Tip {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let height = value["height"].as_u64().context("Invalid height")?;
        let id = value["id"].as_str().context("Invalid id")?;
        let slot = value["slot"].as_u64().context("Invalid slot")?;
        Ok(Tip {
            height: height,
            id: id.to_string(),
            slot: slot,
        })
    }
}

#[derive(Debug)]
pub struct OgmiosRollBackward {
    pub point: Point,
}

impl TryFrom<Value> for OgmiosRollBackward {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let rb = value.as_object().context("Invalid rollbackward")?;
        let point = try_from_point(rb["point"].clone())?;
        Ok(OgmiosRollBackward{point: point})
    }
}

fn try_from_point(value: Value) -> Result<Point, anyhow::Error> {
    if let Some(s) = value.as_str() {
        if s == "origin" {
            Ok(Point::Origin)
        } else {
            Err(anyhow!("Expected 'origin'"))
        }
    } else if let Some(o) = value.as_object() {
        let slot = o["slot"].as_u64().context("Invalid slot")?;
        let id = o["id"].as_str().context("Invalid id")?;
        let id_bytes = hex::decode(id)?;
        Ok(Point::Specific(slot, id_bytes))
    } else {
        Err(anyhow!("Expected str or object"))
    }
}

type Sink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>;
type Stream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub struct OgmiosClient {
    chainsync: ChainsyncClient,
}

pub struct ChainsyncClient {
    write: Sink,
    read: Stream,
}

pub struct ClientError {
    message: String,
}

#[derive(Debug)]
pub struct IntersectResponse {
}

impl Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "ClientError: {}", self.message)
    }
}

impl OgmiosClient {
    pub async fn connect(url: &String) -> Result<Self, ClientError> {
        let request = url.into_client_request().unwrap();
        let (stream, _) = connect_async(request).await.unwrap();
        let (ogmios_write, ogmios_read) = stream.split();
        Ok(OgmiosClient {
            chainsync: ChainsyncClient {
                write: ogmios_write,
                read: ogmios_read,
            }
        })
    }

    pub fn chainsync(&mut self) -> &mut ChainsyncClient {
        &mut self.chainsync
    }

    pub fn abort(&self) {
    }
}

impl ChainsyncClient {
    pub async fn intersect_origin(&mut self) -> Result<IntersectResponse, ClientError> {
        let message = json!({
            "jsonrpc": "2.0",
            "method": "findIntersection",
            "params": json!({
                "points": json!([]),
            }),
        });
        let _ = self.write.send(Message::Text(message.to_string())).await;
        let intersect_response = self.read.next().await.unwrap().unwrap();
        debug!(?intersect_response, "intersect response");
        Ok(IntersectResponse{})
    }
    pub async fn request_next(&mut self) -> Result<OgmiosResponse, ClientError> {
        let message = json!({
            "jsonrpc": "2.0",
            "method": "nextBlock",
        });
        let _ = self.write.send(Message::Text(message.to_string())).await;
        loop {
            let msg = self.read.next().await.unwrap().unwrap();
            debug!(?msg, "nextblock response");
            if let Message::Ping(bytes) = msg {
                let _ = self.write.send(Message::Pong(bytes)).await;
            } else if msg.is_binary() || msg.is_text() {
                let bytes = msg.into_data();
                let v: Result<Value, _> = serde_json::from_slice(&bytes);
                match v {
                    Ok(v) => {
                        let ogmios_response = OgmiosResponse::try_from(v);
                        match ogmios_response {
                            Ok(OgmiosResponse::RollForward(rf)) => {
                                return Ok(OgmiosResponse::RollForward(rf))
                            }
                            Ok(OgmiosResponse::RollBackward(rb)) => {
                                return Ok(OgmiosResponse::RollBackward(rb))
                            }
                            Err(_) => {
                                return Err(ClientError{
                                    message: "Unexpected NextBlock json".to_string(),
                                })
                            }
                        }
                    }
                    Err(_) => {
                        return Err(ClientError{
                            message: "Couldn't decode NextBlock response".to_string(),
                        })
                    }
                }
            } else {
                return Err(ClientError{
                    message: "Unexpected NextBlock response".to_string(),
                })
            }
        }
    }
}
