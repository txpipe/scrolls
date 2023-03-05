use pallas::codec::utils::CborWrap;
use pallas::ledger::primitives::babbage::{DatumOption, PlutusData};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::{Asset, MultiEraBlock, MultiEraTx};
use pallas::ledger::traverse::{MultiEraOutput, OriginalHash};
use serde::Deserialize;
use serde_json::json;

use crate::{crosscut, model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub address: String,
    pub prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
}

pub fn resolve_datum(utxo: &MultiEraOutput, tx: &MultiEraTx) -> Result<PlutusData, ()> {
    match utxo.datum() {
        Some(DatumOption::Data(CborWrap(pd))) => Ok(pd),
        Some(DatumOption::Hash(datum_hash)) => {
            for raw_datum in tx.clone().plutus_data() {
                if raw_datum.original_hash().eq(&datum_hash) {
                    return Ok(raw_datum.clone().unwrap());
                }
            }

            return Err(());
        }
        _ => Err(()),
    }
}

impl Reducer {
    fn get_key_value(&self, utxo: &MultiEraOutput, tx: &MultiEraTx) -> Option<(String, String)> {
        if let Some(address) = utxo.address().map(|addr| addr.to_string()).ok() {
            if address.eq(&self.config.address) {
                let mut data = serde_json::Value::Object(serde_json::Map::new());
                if let Some(datum) = resolve_datum(utxo, tx).ok() {
                    data["datum"] = serde_json::Value::String(hex::encode(
                        datum.encode_fragment().ok().unwrap(),
                    ));
                } else if let Some(DatumOption::Hash(h)) = utxo.datum() {
                    data["datum_hash"] = serde_json::Value::String(hex::encode(h.to_vec()));
                }

                let mut assets: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                for asset in utxo.non_ada_assets() {
                    match asset {
                        Asset::Ada(lovelace_amt) => {
                            assets.insert(
                                String::from("lovelace"),
                                json!({
                                    "unit": "lovelace",
                                    "quantity": format!("{}", lovelace_amt)
                                }),
                            );
                        }
                        Asset::NativeAsset(cs, tkn, amt) => {
                            let unit = format!("{}{}", hex::encode(cs.to_vec()), hex::encode(tkn));
                            assets.insert(
                                unit.clone(),
                                json!({
                                    "unit": unit,
                                    "quantity": format!("{}", amt)
                                }),
                            );
                        }
                    }
                }

                data["amount"] = serde_json::Value::Object(assets);
                return Some((address, data.to_string()));
            }
        }

        None
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let prefix = self.config.prefix.as_deref();
        for tx in block.txs().into_iter() {
            for consumed in tx.consumes().iter().map(|i| i.output_ref()) {
                if let Some(Some(utxo)) = ctx.find_utxo(&consumed).apply_policy(&self.policy).ok() {
                    if let Some((key, value)) = self.get_key_value(&utxo, &tx) {
                        output.send(
                            model::CRDTCommand::set_remove(prefix, &key.as_str(), value).into(),
                        )?;
                    }
                }
            }

            for (_, produced) in tx.produces() {
                if let Some((key, value)) = self.get_key_value(&produced, &tx) {
                    output.send(model::CRDTCommand::set_add(None, &key, value).into())?;
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

        super::Reducer::FullUtxosByAddress(reducer)
    }
}
