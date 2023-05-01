use std::panic;
use pallas::ledger::traverse::{Asset, MultiEraOutput};
use pallas::ledger::traverse::{MultiEraBlock};
use serde::{Deserialize, Serialize};

use crate::{crosscut, model, prelude::*};
use pallas::crypto::hash::Hash;

use bech32::{ToBase32, Variant, Error};
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use log::error;
use pallas::ledger::addresses::{Address, StakeAddress};

use std::collections::HashMap;
use pallas::ledger::addresses::ShelleyPaymentPart::Key;

#[derive(Serialize, Deserialize)]
struct MultiAssetSingleAgg {
    #[serde(rename = "policyId")]
    policy_id: String,
    #[serde(rename = "assetName")]
    asset_name: String,
    quantity: i64,
    fingerprint: String,
}

#[derive(Deserialize, Copy, Clone)]
pub enum Projection {
    Cbor,
    Json,
}

#[derive(Deserialize, Copy, Clone)]
pub enum StakeIndex {
    On,
    Off,
}

#[derive(Deserialize, Copy, Clone)]
pub enum AddressIndex {
    On,
    Off,
}

impl Default for Projection {
    fn default() -> Self {
        Self::Cbor
    }

}

impl Default for StakeIndex {
    fn default() -> Self {
        Self::Off
    }

}

impl Default for AddressIndex {
    fn default() -> Self {
        Self::Off
    }

}

#[derive(Serialize, Deserialize)]
struct PreviousOwnerAgg {
    address: String,
    transferred_out: i64,
}

impl PreviousOwnerAgg {
    fn new(address: &str, transferred_out: u64) -> PreviousOwnerAgg {
        PreviousOwnerAgg {
            address: address.to_string(),
            transferred_out: transferred_out.try_into().unwrap(),
        }

    }

}

fn asset_fingerprint(
    data_list: [&str; 2],
) -> Result<String, Error> {
    let combined_parts = data_list.join("");
    let raw = hex::decode(combined_parts).unwrap();

    let mut hasher = Blake2bVar::new(20).unwrap();
    hasher.update(&raw);
    let mut buf = [0u8; 20];
    hasher.finalize_variable(&mut buf).unwrap();
    let base32_combined = buf.to_base32();
    bech32::encode("asset", base32_combined, Variant::Bech32)
}

#[derive(Deserialize, Copy, Clone)]
pub enum AggrType {
    Epoch,

}

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<crosscut::filters::Predicate>,
    pub stake_index: Option<StakeIndex>,
    pub address_index: Option<AddressIndex>,
    pub projection: Option<Projection>,
}

pub struct Reducer {
    config: Config,
    chain: crosscut::ChainWellKnownInfo,
    policy: RuntimePolicy,
    time: crosscut::time::NaiveProvider,
}

impl Reducer {
    fn stake_or_address_from_address(&self, address: &Address) -> String {
        match address {
            Address::Shelley(s) => match StakeAddress::try_from(s.clone()).ok() {
                Some(x) => x.to_bech32().unwrap_or(String::new()),
                _ => address.to_bech32().unwrap_or(String::new()),
            },
            Address::Byron(_) => address.to_bech32().unwrap_or(String::new()),
            Address::Stake(_) => address.to_bech32().unwrap_or(String::new()),
        }
    }

    fn process_produced_txo(
        &self,
        config: &Config,
        tx_output: &MultiEraOutput,
        timestamp: &u64,
        output: &mut super::OutputPort,
        stake_or_address: String,
    ) -> Result<(), gasket::error::Error> {
        let mut fingerprint_tallies: HashMap<String, i64> = HashMap::new();

        for asset in tx_output.assets() {
            if let Asset::NativeAsset(policy_id, asset_name, quantity) = asset {
                let asset_result = panic::catch_unwind(|| hex::encode(asset_name));
                if let Ok(asset_name) = asset_result {
                    model::CRDTCommand::HashCounter(
                        format!("asset-qty.{}.{}.{}", self.config.key_prefix.as_deref().unwrap_or_default(), stake_or_address, fingerprint),
                    )

                    if !fingerprint.is_empty() && !stake_or_address.is_empty() {
                        *fingerprint_tallies.entry(fingerprint).or_insert(quantity as i64) += quantity as i64;
                    }

                }

            };

        }

        for (fingerprint, quantity) in fingerprint_tallies {
            let total_asset_count = model::CRDTCommand::PNCounter(
                format!("asset-qty.{}.{}.{}", self.config.key_prefix.as_deref().unwrap_or_default(), stake_or_address, fingerprint),
                quantity
            );

            output.send(total_asset_count.into())?;

        }

        Ok(())
    }

    fn process_spent_txo(
        &mut self,
        tx_input: &MultiEraOutput,
        timestamp: &u64,
        tx_hash: &str,
        tx_index: i64,
        output: &mut super::OutputPort,
        stake_or_address: String,
    ) -> Result<(), gasket::error::Error> {
        // for asset in tx_input.assets() {
        //     if let Asset::NativeAsset(policy_id, asset_name, quantity) = asset {
        //         let asset_result = panic::catch_unwind(|| hex::encode(asset_name));
        //         if let Ok(asset_name) = asset_result {
        //             let (fingerprint, _) = MultiAssetSingleAgg::new(
        //                 policy_id,
        //                 asset_name.as_str(),
        //                 quantity,
        //                 tx_hash,
        //                 tx_index,
        //             ).unwrap();
        //
        //             if !fingerprint.is_empty() {
        //                 let total_asset_count = model::CRDTCommand::PNCounter(
        //                     format!("asset-qty.{}.{}.{}", self.config.key_prefix.as_deref().unwrap_or_default(), stake_or_address, fingerprint),
        //                     -1 * quantity as i64
        //                 );
        //
        //                 if let Ok(total_asset_count_message) = total_asset_count.try_into() {
        //                     output.send(total_asset_count_message)?;
        //                 }
        //
        //             }
        //
        //         }
        //
        //     };
        //
        // }

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for (tx_index, tx) in block.txs().into_iter().enumerate() {
            let timestamp = self.time.slot_to_wallclock(block.slot().to_owned());
            for (_, meo) in tx.produces() {
                if let Ok(address) = meo.address() {
                    self.process_produced_txo(
                        &self.config,
                        &meo,
                        &timestamp,
                        output,
                        self.stake_or_address_from_address(&address),
                    )?;
                }

            }

            // for (_, mei) in ctx.find_consumed_txos(&tx, &self.policy).unwrap_or_default() {
            //     if let Ok(address) = mei.address() {
            //         let stake_or_address = self.stake_or_address_from_address(&address);
            //         if stake_or_address.len() > 0 {
            //             self.process_spent_txo(&mei, &timestamp, hex::encode(tx.hash()).as_str(), tx_index.try_into().unwrap(), output, stake_or_address);
            //         }
            //
            //     }
            //
            // }

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
        let reducer = Reducer {
            config: self,
            chain: chain.clone(),
            policy: policy.clone(),
            time: crosscut::time::NaiveProvider::new(chain.clone()),
        };

        super::Reducer::MultiAssetBalances(reducer)
    }

}
