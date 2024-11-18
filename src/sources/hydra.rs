use anyhow::{Context, Result};
use serde_json::Value;

use pallas_primitives::babbage::{Header};
use pallas_primitives::conway::{VrfCert};
use pallas_primitives::conway::{HeaderBody, OperationalCert, PseudoBlock, Tx};
use pallas_codec::utils::{Bytes, KeepRaw, KeyValuePairs, MaybeIndefArray, Nullable};
use pallas_crypto::hash::Hash;

// src/Hydra/Events.hs 'StateEvent'
// This is the type sent to EventSinks.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Event {
    pub event_id: u64,
    pub state_changed: StateChanged,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TransactionReceived {
    pub tx: TransactionReceivedTx,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TransactionReceivedTx {
    pub cbor_hex: String,
    pub description: String,
    pub tx_id: String,
    pub r#type: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum StateChanged {
    TransactionReceived(TransactionReceived),
    SnapshotConfirmed(SnapshotConfirmed),
    Unimplemented,
}


impl TryFrom<Value> for Event {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let event_id = value["eventId"].as_u64().context("Invalid eventId")?;
        let state_changed = StateChanged::try_from(value["stateChanged"].clone())?;
        Ok(Event{
            event_id,
            state_changed,
        })
    }
}

impl TryFrom<Value> for TransactionReceived {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let tx = TransactionReceivedTx::try_from(value["tx"].clone())?;
        Ok(TransactionReceived{
            tx: tx,
        })
    }
}

impl TryFrom<Value> for TransactionReceivedTx {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let cbor_hex = value["cborHex"].as_str().context("Invalid cborHex")?;
        let description = value["description"].as_str().context("Invalid description")?;
        let tx_id = value["txId"].as_str().context("Invalid txId")?;
        let r#type = value["type"].as_str().context("Invalid type")?;
        Ok(TransactionReceivedTx{
            cbor_hex: cbor_hex.to_string(),
            description: description.to_string(),
            tx_id: tx_id.to_string(),
            r#type: r#type.to_string(),
        })
    }
}

impl TryFrom<Value> for StateChanged {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let tag = value["tag"].as_str().context("Invalid tag")?;

        match tag {
            "SnapshotConfirmed" => {
                SnapshotConfirmed::try_from(value).map(StateChanged::SnapshotConfirmed)
            }
            "TransactionReceived" => {
                TransactionReceived::try_from(value).map(StateChanged::TransactionReceived)
            }
            _ => Ok(StateChanged::Unimplemented),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SnapshotConfirmed {
    pub confirmed_transactions: Vec<String>,
}

impl TryFrom<Value> for SnapshotConfirmed {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let snapshot = value["snapshot"].as_object().context("Invalid snapshot")?;
        let confirmed_transactions = snapshot["confirmedTransactions"]
            .as_array()
            .context("Invalid confirmedTransactions")?
            .iter()
            .map(|s| {
                 s.as_str()
                    .map(|s| s.into())
                    .context("invalid confirmedTransaction")
            })
            .collect::<Result<Vec<String>>>()?;
        Ok(SnapshotConfirmed {
            confirmed_transactions,
        })
    }
}

pub enum HydraWsMessage {
    TxValid((String, String)),
    TxInvalid((String, String)),
    SnapshotConfirmed(SnapshotConfirmed),
    TransactionReceived(TransactionReceived),
    Unimplemented(()),
}

impl TryFrom<Value> for HydraWsMessage {
    type Error = anyhow::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let tag = value["tag"].as_str().context("Invalid tag")?;
        if tag == "TxValid" {
            let transaction = value["transaction"].as_object().context("Invalid transaction")?;
            let tx_id = transaction["txId"].as_str().context("Invalid txId")?;
            let tx_cbor = transaction["cborHex"].as_str().context("Invalid cborHex")?;
            return Ok(HydraWsMessage::TxValid((tx_id.to_string(), tx_cbor.to_string())));
        } else if tag == "TxInvalid" {
            let transaction = value["transaction"].as_object().context("Invalid transaction")?;
            let tx_id = transaction["txId"].as_str().context("Invalid txId")?;
            let validation_error = value["validationError"].as_object().context("Invalid validationError")?;
            let reason = validation_error["reason"].as_str().context("Invalid reason")?;
            return Ok(HydraWsMessage::TxInvalid((tx_id.to_string(), reason.to_string())));
        } else if tag == "SnapshotConfirmed" {
            return SnapshotConfirmed::try_from(value).map(HydraWsMessage::SnapshotConfirmed)
        } else if tag == "TransactionReceived" {
            return TransactionReceived::try_from(value).map(HydraWsMessage::TransactionReceived)
        } else {
            return Ok(HydraWsMessage::Unimplemented(()));
        }
    }
}

pub struct ConwayBlock {
    pub cbor: Vec<u8>,
    pub slot: u64,
    pub hash: Vec<u8>,
}

pub fn make_conway_block<'a>(transactions: Vec<String>, slot: u64) -> Result<ConwayBlock, anyhow::Error> {
    let mut bodies = vec![];
    let mut witnesses = vec![];
    let mut aux = vec![];
    let mut index = 0u32;
    for tx in &transactions {
        let cbor_raw = hex::decode(&tx)?;
        let pallas_tx: Tx = minicbor::decode(&cbor_raw)?;
        bodies.push(pallas_tx.transaction_body.clone());
        witnesses.push(pallas_tx.transaction_witness_set.clone());
        if let Nullable::Some(x) = pallas_tx.auxiliary_data.clone() {
            aux.push((index, x));
        }
        index += 1;
    }
    let minted_header_body = HeaderBody {
        block_number: 0u64,
        slot: slot,
        prev_hash: None,
        issuer_vkey: Bytes::from(vec![]),
        vrf_vkey: Bytes::from(vec![]),
        vrf_result: VrfCert(Bytes::from(vec![]), Bytes::from(vec![])),
        block_body_size: 0u64,
        block_body_hash: Hash::from([0; 32]),
        operational_cert: OperationalCert {
            operational_cert_hot_vkey: Bytes::from(vec![]),
            operational_cert_sequence_number: 0u64,
            operational_cert_kes_period: 0u64,
            operational_cert_sigma: Bytes::from(vec![]),
        },
        protocol_version: (0, 0),
    };
    let slot = minted_header_body.slot;
    let hash = minted_header_body.block_body_hash.to_vec();
    let header = Header {
        header_body: minted_header_body,
        body_signature: Bytes::from(vec![]),
    };
    let header_bytes = minicbor::to_vec(header)?;
    let raw_header: KeepRaw<Header> = minicbor::decode(&header_bytes)?;
    let block = PseudoBlock {
        header: raw_header,
        transaction_bodies: MaybeIndefArray::Def(bodies),
        transaction_witness_sets: MaybeIndefArray::Def(witnesses),
        auxiliary_data_set: KeyValuePairs::Def(aux),
        invalid_transactions: None,
    };
    let cbor = minicbor::to_vec(block)?;
    Ok(ConwayBlock {
        cbor: cbor,
        slot: slot,
        hash: hash,
    })
}


