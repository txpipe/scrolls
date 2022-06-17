use crosscut::policies::*;
use gasket::error::AsWorkError;
use pallas::crypto::hash::Hash;
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
    pub policy: Option<ReducerPolicy>,
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
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            let tx_hash = tx.hash();

            for input in tx.inputs().iter().filter_map(|i| i.output_ref()) {
                let inbound_tx = ctx
                    .find_ref_tx(input.tx_id())
                    .apply_policy(&self.config.policy)
                    .or_work_err()?;

                if let Some(inbound_tx) = inbound_tx {
                    let output = inbound_tx
                        .output_at(input.tx_index() as usize)
                        .ok_or(crate::Error::ledger("output index not found in tx"))
                        .or_work_err()?;

                    let address = output.address(&self.address_hrp);
                    log::error!("{}", address);
                }
            }

            for (idx, tx_output) in tx.outputs().iter().enumerate() {
                let address = tx_output.address(&self.address_hrp);
                self.send_set_add(&address, tx_hash, idx, output)?;
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
