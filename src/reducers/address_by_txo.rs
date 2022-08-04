use gasket::error::AsWorkError;
use pallas::crypto::hash::Hash;
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::prelude::*;
use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<crosscut::filters::Predicate>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
}

impl Reducer {
    fn send(
        &mut self,
        slot: u64,
        address: &str,
        tx_hash: Hash<32>,
        output_idx: usize,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let crdt = model::CRDTCommand::last_write_wins(
            self.config.key_prefix.as_deref(),
            &format!("{}#{}", tx_hash, output_idx),
            address.to_string(),
            slot,
        );

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block(
        &mut self,
        block: &MultiEraBlock,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let slot = block.slot();

        for tx in block.txs() {
            if filter_matches!(self, block, &tx, ctx) {
                let tx_hash = tx.hash();

                for (output_idx, tx_out) in tx.outputs().iter().enumerate() {
                    let address = tx_out.address().map(|x| x.to_string()).or_panic()?;

                    self.send(slot, &address, tx_hash, output_idx, output)?;
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, policy: &crosscut::policies::RuntimePolicy) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            policy: policy.clone(),
        };

        super::Reducer::AddressByTxo(reducer)
    }
}
