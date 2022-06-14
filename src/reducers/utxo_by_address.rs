use pallas::crypto::hash::Hash;
use pallas::ledger::traverse::MultiEraBlock;
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
        output_idx: usize,
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

        let member = format!("{}#{}", tx_hash, output_idx);
        let crdt = model::CRDTCommand::TwoPhaseSetAdd(key, member);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            let tx_hash = tx.hash();

            for tx_output in tx.outputs() {
                let address = tx_output.address(&self.address_hrp);
                self.send_set_add(&address, tx_hash, 0, output)?;
            }
        }

        Ok(())
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
