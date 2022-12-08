use gasket::error::AsWorkError;
use pallas::crypto::hash::Hash;
use pallas::ledger::traverse::Asset;
use pallas::ledger::traverse::{MultiEraBlock, MultiEraTx};
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
}

impl Reducer {
    fn process_asset(
        &mut self,
        tx: &MultiEraTx,
        txo_idx: u64,
        policy: Hash<28>,
        asset: Vec<u8>,
        delta: i64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let tx_hash = tx.hash();
        let prefix = self.config.key_prefix.as_deref();
        let key = &format!("{}{}", policy, hex::encode(asset));
        let member = format!("{}#{}", tx_hash, txo_idx);

        let crdt = match delta {
            x if x < 0 => model::CRDTCommand::sorted_set_remove(prefix, key, member, delta),
            _ => model::CRDTCommand::sorted_set_add(prefix, key, member, delta),
        };

        output.send(crdt.into())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            for (idx, txo) in ctx.find_consumed_txos(&tx, &self.policy).or_panic()? {
                for asset in txo.assets() {
                    if let Asset::NativeAsset(policy, asset, delta) = asset {
                        self.process_asset(&tx, idx, policy, asset, -1 * delta as i64, output)?;
                    }
                }
            }

            for (idx, txo) in tx.produces() {
                for asset in txo.assets() {
                    if let Asset::NativeAsset(policy, asset, delta) = asset {
                        self.process_asset(&tx, idx as u64, policy, asset, delta as i64, output)?;
                    }
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

        super::Reducer::UtxosByAsset(reducer)
    }
}
