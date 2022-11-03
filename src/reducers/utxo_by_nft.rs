use pallas::crypto::hash::Hash;
use pallas::ledger::traverse::{Asset, MultiEraOutput};
use pallas::ledger::traverse::{MultiEraBlock, MultiEraTx};
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
}

impl Reducer {
    fn process_received_asset(
        &mut self,
        tx: &MultiEraTx,
        txo_idx: usize,
        policy: Hash<28>,
        asset: Vec<u8>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let tx_hash = tx.hash();

        let crdt = model::CRDTCommand::any_write_wins(
            self.config.key_prefix.as_deref(),
            format!("{}{}", policy, hex::encode(asset)),
            format!("{}#{}", tx_hash, txo_idx),
        );

        output.send(crdt.into())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        _ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            for (idx, txo) in tx.produces() {
                for asset in txo.assets() {
                    if let Asset::NativeAsset(policy, asset, quantity) = asset {
                        // This check is to avoid indexing fungible tokens. This method will have
                        // several false positives, but it's fast and provides a good approximation.
                        if quantity == 1 {
                            self.process_received_asset(&tx, idx, policy, asset, output)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, _policy: &crosscut::policies::RuntimePolicy) -> super::Reducer {
        let reducer = Reducer { config: self };

        super::Reducer::UtxoByNft(reducer)
    }
}
