use std::str::FromStr;

use pallas_crypto::hash::Hash;
use pallas_traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub policy_ids_hex: Option<Vec<String>>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    policy_ids: Option<Vec<Hash<28>>>,
}

impl Reducer {
    fn is_policy_id_accepted(&self, policy_id: &Hash<28>) -> bool {
        return match &self.policy_ids {
            Some(pids) => pids.contains(&policy_id),
            None => true,
        };
    }

    fn process_asset(
        &mut self,
        policy: &Hash<28>,
        asset: &[u8],
        qty: i64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if !self.is_policy_id_accepted(&policy) {
            return Ok(());
        }

        let asset_id = &format!("{}{}", policy, hex::encode(asset));

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, asset_id),
            None => format!("{}.{}", "supply_by_asset".to_string(), asset_id),
        };

        let crdt = model::CRDTCommand::PNCounter(key, qty);

        output.send(crdt.into())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            for mint in tx.mints() {
                for asset in mint.assets() {
                    self.process_asset(
                        mint.policy(),
                        asset.name(),
                        asset.mint_coin().unwrap_or_default(),
                        output,
                    )?;
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, policy: &crosscut::policies::RuntimePolicy) -> super::Reducer {
        let policy_ids: Option<Vec<Hash<28>>> = match &self.policy_ids_hex {
            Some(pids) => {
                let ps = pids
                    .iter()
                    .map(|pid| Hash::<28>::from_str(pid).expect("invalid policy_id"))
                    .collect();

                Some(ps)
            }
            None => None,
        };

        let reducer = Reducer {
            config: self,
            policy: policy.clone(),
            policy_ids,
        };

        super::Reducer::SupplyByAsset(reducer)
    }
}
