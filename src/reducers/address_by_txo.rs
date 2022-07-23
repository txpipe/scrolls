use gasket::error::AsWorkError;
use pallas::crypto::hash::Hash;
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::model;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
}

pub struct Reducer {
    config: Config,
}

impl Reducer {
    fn send_set_add(
        &mut self,
        slot: u64,
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
            Some(prefix) => format!("{}.{}#{}", prefix, tx_hash, output_idx),
            None => format!("{}#{}", tx_hash, output_idx),
        };

        let crdt = model::CRDTCommand::LastWriteWins(key, address.to_string(), slot);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block(
        &mut self,
        block: &MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let slot = block.slot();

        for tx in block.txs() {
            let tx_hash = tx.hash();

            for (output_idx, tx_out) in tx.outputs().iter().enumerate() {
                let address = tx_out.address().map(|x| x.to_string()).or_panic()?;

                self.send_set_add(slot, &address, tx_hash, output_idx, output)?;
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let reducer = Reducer { config: self };

        super::Reducer::AddressByTxo(reducer)
    }
}
