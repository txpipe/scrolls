use pallas::codec::utils::CborWrap;
use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::babbage::{PlutusData, PseudoDatumOption};
use pallas::ledger::primitives::Fragment;
use pallas::ledger::traverse::{
    MultiEraBlock, MultiEraOutput, MultiEraPolicyAssets, MultiEraTx, OriginalHash,
};
use serde::Deserialize;
use serde_json::json;

use crate::framework::model::CRDTCommand;
use crate::framework::{model, Error};

use super::{ReducerConfigTrait, ReducerTrait};

#[derive(Deserialize)]
pub struct Config {
    pub filter: Vec<String>,
    pub prefix: Option<String>,
    pub address_as_key: Option<bool>,
}
impl ReducerConfigTrait for Config {
    fn plugin(self) -> Box<dyn ReducerTrait> {
        let reducer = Reducer { config: self };
        Box::new(reducer)
    }
}

pub struct Reducer {
    config: Config,
    // policy: crosscut::policies::RuntimePolicy,
}

pub fn resolve_datum(utxo: &MultiEraOutput, tx: &MultiEraTx) -> Option<PlutusData> {
    if let Some(datum) = utxo.datum() {
        return match datum {
            PseudoDatumOption::Data(CborWrap(pd)) => Some(pd.unwrap()),
            PseudoDatumOption::Hash(datum_hash) => {
                for raw_datum in tx.clone().plutus_data() {
                    if raw_datum.original_hash().eq(&datum_hash) {
                        return Some(raw_datum.clone().unwrap());
                    }
                }

                return None;
            }
        };
    }

    None
}

impl Reducer {
    fn get_key_value(
        &self,
        utxo: &MultiEraOutput,
        tx: &MultiEraTx,
        output_ref: &(Hash<32>, u64),
    ) -> Option<(String, String)> {
        if let Some(address) = utxo.address().map(|addr| addr.to_string()).ok() {
            if self.config.filter.iter().any(|addr| address.eq(addr)) {
                let mut data = serde_json::Value::Object(serde_json::Map::new());
                let address_as_key = self.config.address_as_key.unwrap_or(false);
                let key: String;

                if address_as_key {
                    key = address;
                    data["tx_hash"] = serde_json::Value::String(hex::encode(output_ref.0.to_vec()));
                    data["output_index"] =
                        serde_json::Value::from(serde_json::Number::from(output_ref.1));
                } else {
                    key = format!("{}#{}", hex::encode(output_ref.0.to_vec()), output_ref.1);
                    data["address"] = serde_json::Value::String(address);
                }

                if let Some(datum) = resolve_datum(utxo, tx) {
                    data["datum"] = serde_json::Value::String(hex::encode(
                        datum.encode_fragment().ok().unwrap(),
                    ));
                } else if let Some(PseudoDatumOption::Hash(h)) = utxo.datum() {
                    data["datum_hash"] = serde_json::Value::String(hex::encode(h.to_vec()));
                }

                let mut assets: Vec<serde_json::Value> = vec![json!({
                    "unit": "lovelace",
                    "quantity": format!("{}", utxo.lovelace_amount())
                })];

                for asset in utxo.non_ada_assets() {
                    match asset {
                        MultiEraPolicyAssets::AlonzoCompatibleOutput(_, pairs) => assets.append(
                            &mut pairs
                                .iter()
                                .map(|(name, amount)| {
                                    json!({
                                        "unit": name,
                                        "quantity": format!("{}", amount)
                                    })
                                })
                                .collect::<Vec<serde_json::Value>>(),
                        ),
                        _ => todo!(),
                    }
                }

                data["amount"] = serde_json::Value::Array(assets);
                return Some((key, data.to_string()));
            }
        }

        None
    }
}

#[async_trait::async_trait]
impl ReducerTrait for Reducer {
    async fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
    ) -> Result<Vec<CRDTCommand>, Error> {
        let prefix = self.config.prefix.as_deref();
        let mut commands: Vec<CRDTCommand> = Vec::new();

        for tx in block.txs().into_iter() {
            for consumed in tx.consumes().iter().map(|i| i.output_ref()) {
                if let Some(utxo) = ctx.find_utxo(&consumed).ok() {
                    if let Some((key, value)) =
                        self.get_key_value(&utxo, &tx, &(consumed.hash().clone(), consumed.index()))
                    {
                        commands.push(CRDTCommand::set_remove(prefix, &key.as_str(), value));
                    }
                }
            }

            for (index, produced) in tx.produces() {
                let output_ref = (tx.hash().clone(), index as u64);
                if let Some((key, value)) = self.get_key_value(&produced, &tx, &output_ref) {
                    commands.push(CRDTCommand::set_add(None, &key, value));
                }
            }
        }

        Ok(commands)
    }
}
