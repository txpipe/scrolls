use pallas::ledger::addresses::{Address, StakeAddress};
use pallas::ledger::traverse::{Asset, MultiEraOutput};
use pallas::ledger::traverse::{MultiEraBlock, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};
use pallas::crypto::hash::Hash;

use crate::crosscut::epochs::block_epoch;
use std::str::FromStr;

#[derive(Deserialize, Copy, Clone)]
pub enum AggrType {
    Epoch,
}

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<crosscut::filters::Predicate>,
    pub aggr_by: Option<AggrType>,
    pub policy_ids_hex: Option<Vec<String>>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    chain: crosscut::ChainWellKnownInfo,
    policy_ids: Option<Vec<Hash<28>>>,
}

fn any_address_to_stake_bech32(address: Address) -> Option<String> {
    match address {
        Address::Shelley(s) => match StakeAddress::try_from(s).ok() {
            Some(x) => x.to_bech32().ok(),
            _ => None,
        },
        Address::Byron(_) => None,
        Address::Stake(_) => None,
    }
}

impl Reducer {
    fn config_key(&self, stake_address: String, epoch_no: u64) -> String {
        let def_key_prefix = "assets_by_stake_key";

        match &self.config.aggr_by {
            Some(aggr_type) if matches!(aggr_type, AggrType::Epoch) => {
                return match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}.{}", prefix, stake_address, epoch_no),
                    None => format!("{}.{}", def_key_prefix.to_string(), stake_address),
                };
            }
            _ => {
                return match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}", prefix, stake_address),
                    None => format!("{}.{}", def_key_prefix.to_string(), stake_address),
                };
            }
        };
    }

    fn is_policy_id_accepted(&self, policy_id: &Hash<28>) -> bool {
        return match &self.policy_ids {
            Some(pids) => pids.contains(&policy_id),
            None => true,
        };
    }

    fn process_consumed_txo(
        &mut self,
        ctx: &model::BlockContext,
        input: &OutputRef,
        epoch_no: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let utxo = ctx.find_utxo(input).apply_policy(&self.policy).or_panic()?;

        let utxo = match utxo {
            Some(x) => x,
            None => return Ok(()),
        };

        let address = utxo.address().or_panic()?;
        let stake_address = any_address_to_stake_bech32(address);

        let stake_address = match stake_address {
            Some(x) => x,
            None => return Ok(()),
        };

        for asset in utxo.assets() {
            match asset {
                Asset::NativeAsset(policy_id, _, quantity) => {
                    if self.is_policy_id_accepted(&policy_id) {
                        let subject = asset.subject();
                        let key = self.config_key(stake_address.clone(), epoch_no);
                        let delta = quantity as i64 * (-1);

                        let crdt =
                            model::CRDTCommand::SortedSetRemove(key, subject, delta);

                        output.send(gasket::messaging::Message::from(crdt))?;
                    }
                }
                _ => (),
            };
        }

        Ok(())
    }

    fn process_produced_txo(
        &mut self,
        tx_output: &MultiEraOutput,
        epoch_no: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let address = tx_output.address().or_panic()?;
        let stake_address = any_address_to_stake_bech32(address);

        let stake_address = match stake_address {
            Some(x) => x,
            None => return Ok(()),
        };

        for asset in tx_output.assets() {
            match asset {
                Asset::NativeAsset(policy_id, _, quantity) => {
                    if self.is_policy_id_accepted(&policy_id) {
                        let subject = asset.subject();
                        let key = self.config_key(stake_address.clone(), epoch_no);
                        let delta = quantity as i64;

                        let crdt =
                            model::CRDTCommand::SortedSetAdd(key, subject, delta);

                        output.send(gasket::messaging::Message::from(crdt))?;
                    }
                }
                _ => {}
            };
        }

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            if filter_matches!(self, block, &tx, ctx) {
                let epoch_no = block_epoch(&self.chain, block);

                for consumed in tx.consumes().iter().map(|i| i.output_ref()) {
                    self.process_consumed_txo(&ctx, &consumed, epoch_no, output)?;
                }

                for (_, meo) in tx.produces() {
                    self.process_produced_txo(&meo, epoch_no, output)?;
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        policy: &crosscut::policies::RuntimePolicy,
    ) -> super::Reducer {
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
            chain: chain.clone(),
            policy: policy.clone(),
            policy_ids: policy_ids.clone(),
        };

        super::Reducer::AssetsByStakeKey(reducer)
    }
}