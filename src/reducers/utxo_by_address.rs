use gasket::error::AsWorkError;
use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::{alonzo, byron, ToHash};
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
}

pub struct Reducer {
    config: Config,
    address_hrp: String,
}

impl Reducer {
    fn send_set_add(
        &mut self,
        address: &str,
        tx_hash: Hash<32>,
        tx_idx: usize,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if let Some(addresses) = &self.config.filter {
            if let Err(_) = addresses.binary_search(&address.to_string()) {
                return Ok(());
            }
        }

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, address),
            None => address.to_string(),
        };

        let member = format!("{}:{}", tx_hash, tx_idx);
        let crdt = model::CRDTCommand::TwoPhaseSetAdd(key, member);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    fn reduce_byron_tx(
        &mut self,
        tx: &byron::TxPayload,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let tx_hash = tx.transaction.to_hash();

        tx.transaction
            .outputs
            .iter()
            .enumerate()
            .map(move |(tx_idx, tx)| {
                let address = tx.address.to_addr_string().or_work_err()?;
                self.send_set_add(&address, tx_hash, tx_idx, output)
            })
            .collect()
    }

    fn reduce_alonzo_compatible_tx(
        &mut self,
        tx: &alonzo::TransactionBody,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let tx_hash = tx.to_hash();

        tx.iter()
            .filter_map(|b| match b {
                alonzo::TransactionBodyComponent::Outputs(o) => Some(o),
                _ => None,
            })
            .flat_map(|o| o.iter())
            .enumerate()
            .map(move |(tx_idx, tx_output)| {
                let address = tx_output
                    .to_bech32_address(&self.address_hrp)
                    .or_work_err()?;
                self.send_set_add(&address, tx_hash, tx_idx, output)
            })
            .collect()
    }

    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(byron::Block::MainBlock(x)) => x
                .body
                .tx_payload
                .iter()
                .map(|tx| self.reduce_byron_tx(tx, output))
                .collect(),
            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(x) => {
                x.1.transaction_bodies
                    .iter()
                    .map(|tx| self.reduce_alonzo_compatible_tx(tx, output))
                    .collect()
            }
        }
    }
}

impl Config {
    pub fn plugin(self, chain: &crosscut::ChainWellKnownInfo) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            address_hrp: chain.address_hrp.clone(),
        };

        super::Reducer::UtxoByAddress(reducer)
    }
}
