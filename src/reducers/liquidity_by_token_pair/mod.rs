use lazy_static::__Deref;
use pallas::{
    codec::utils::CborWrap,
    ledger::{
        primitives::babbage::DatumOption,
        traverse::{Asset, MultiEraBlock, MultiEraOutput},
    },
};
use serde::Deserialize;

pub mod minswap;
pub mod model;
pub mod sundaeswap;
pub mod utils;
pub mod wingriders;

use crate::{crosscut, prelude::*};

use self::{
    model::{PoolAsset, PoolDatum, TokenPair},
    sundaeswap::SundaePoolDatum,
    utils::contains_currency_symbol,
    wingriders::WingriderPoolDatum,
};

#[derive(Deserialize)]
pub struct Config {
    pub pool_prefix: Option<String>,
    pub dex_prefix: Option<String>,
    pub pool_currency_symbol: String,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
}

fn get_asset_amount(asset: &PoolAsset, assets: &Vec<Asset>) -> Option<u64> {
    match asset {
        PoolAsset::Ada => {
            for a in assets {
                if let Asset::Ada(amt) = a {
                    return Some(*amt);
                }
            }
        }
        PoolAsset::AssetClass(ppid, ptkn) => {
            let pid_str = hex::encode(ppid.deref().to_vec());
            let tkn_str = hex::encode(ptkn.deref());
            for a in assets {
                if let Asset::NativeAsset(pid, tkn, amt) = a {
                    if hex::encode(pid.deref()).eq(&pid_str) && hex::encode(tkn).eq(&tkn_str) {
                        return Some(*amt);
                    }
                }
            }
        }
    }

    None
}

impl Reducer {
    fn get_key_value_pair(&self, utxo: &MultiEraOutput) -> Result<(String, String), ()> {
        if !contains_currency_symbol(
            &self.config.pool_currency_symbol,
            utxo.non_ada_assets().as_ref(),
        ) {
            return Err(());
        }

        // Try to get embedded datum & decode datum data to supported pool datum
        if let Some(DatumOption::Data(CborWrap(pd))) = utxo.datum() {
            if let Some(pool_datum) = PoolDatum::try_from(&pd).ok() {
                let assets = utxo.assets();
                return match pool_datum {
                    PoolDatum::Minswap(TokenPair { coin_a, coin_b })
                    | PoolDatum::Wingriders(WingriderPoolDatum { coin_a, coin_b }) => {
                        if let (Some(coin_a_amt), Some(coin_b_amt), Some(key)) = (
                            get_asset_amount(&coin_a, &assets),
                            get_asset_amount(&coin_b, &assets),
                            TokenPair { coin_a, coin_b }.key(),
                        ) {
                            return Ok((key, format!("{}:{}", coin_a_amt, coin_b_amt)));
                        }

                        Err(())
                    }
                    PoolDatum::Sundaeswap(SundaePoolDatum {
                        coin_a,
                        coin_b,
                        fee,
                    }) => {
                        if let (Some(coin_a_amt), Some(coin_b_amt), Some(key)) = (
                            get_asset_amount(&coin_a, &assets),
                            get_asset_amount(&coin_b, &assets),
                            TokenPair { coin_a, coin_b }.key(),
                        ) {
                            return Ok((key, format!("{}:{}:{}", coin_a_amt, coin_b_amt, fee)));
                        }

                        Err(())
                    }
                };
            }
        }
        Err(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &crate::model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let pool_prefix = self.config.pool_prefix.as_deref();
        for tx in block.txs().into_iter() {
            for consumed in tx.consumes().iter().map(|i| i.output_ref()) {
                if let Some(utxo) = ctx
                    .find_utxo(&consumed)
                    .apply_policy(&self.policy)
                    .or_panic()?
                {
                    if let Some((k, v)) = self.get_key_value_pair(&utxo).ok() {
                        output.send(
                            crate::model::CRDTCommand::set_remove(pool_prefix, &k, v).into(),
                        )?;
                    }
                }
            }

            for (_, produced) in tx.produces() {
                if let Some((k, v)) = self.get_key_value_pair(&produced).ok() {
                    output.send(crate::model::CRDTCommand::set_add(pool_prefix, &k, v).into())?;
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
        super::Reducer::LiquidityByTokenPair(reducer)
    }
}
