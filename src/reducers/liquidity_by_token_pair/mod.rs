use lazy_static::__Deref;
use pallas::ledger::traverse::{Asset, MultiEraBlock, MultiEraOutput, MultiEraTx};
use serde::Deserialize;

pub mod minswap;
pub mod model;
pub mod sundaeswap;
pub mod utils;
pub mod wingriders;

use crate::{crosscut, prelude::*};

use self::{
    model::{LiquidityPoolDatum, PoolAsset, TokenPair},
    sundaeswap::SundaePoolDatum,
    utils::{build_key_value_pair, contains_currency_symbol, resolve_datum},
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
    fn get_key_value_pair(
        &self,
        tx: &MultiEraTx,
        utxo: &MultiEraOutput,
    ) -> Result<(String, String), ()> {
        if !contains_currency_symbol(&self.config.pool_currency_symbol, &utxo.non_ada_assets()) {
            return Err(());
        }

        // Get embedded datum for txIns or inline datums if applicable
        let plutus_data = resolve_datum(utxo, tx)?;
        // Try to decode datum as known liquidity pool datum
        let pool_datum = LiquidityPoolDatum::try_from(&plutus_data)?;
        let assets = utxo.assets();
        match pool_datum {
            LiquidityPoolDatum::Minswap(TokenPair { coin_a, coin_b })
            | LiquidityPoolDatum::Wingriders(WingriderPoolDatum { coin_a, coin_b }) => {
                let coin_a_amt_opt = get_asset_amount(&coin_a, &assets);
                let coin_b_amt_opt = get_asset_amount(&coin_b, &assets);
                return build_key_value_pair(
                    &TokenPair { coin_a, coin_b },
                    &self.config.dex_prefix,
                    coin_a_amt_opt,
                    coin_b_amt_opt,
                    None,
                )
                .ok_or(());
            }
            LiquidityPoolDatum::Sundaeswap(SundaePoolDatum {
                coin_a,
                coin_b,
                fee,
            }) => {
                let coin_a_amt_opt = get_asset_amount(&coin_a, &assets);
                let coin_b_amt_opt = get_asset_amount(&coin_b, &assets);
                return build_key_value_pair(
                    &TokenPair { coin_a, coin_b },
                    &self.config.dex_prefix,
                    coin_a_amt_opt,
                    coin_b_amt_opt,
                    Some(fee),
                )
                .ok_or(());
            }
        };
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
                if let Some(Some(utxo)) = ctx.find_utxo(&consumed).apply_policy(&self.policy).ok() {
                    if let Some((k, v)) = self.get_key_value_pair(&tx, &utxo).ok() {
                        output.send(
                            crate::model::CRDTCommand::set_remove(pool_prefix, &k, v).into(),
                        )?;
                    }
                }
            }

            for (_, produced) in tx.produces() {
                if let Some((k, v)) = self.get_key_value_pair(&tx, &produced).ok() {
                    output.send(
                        crate::model::CRDTCommand::set_add(pool_prefix, &k.as_str(), v).into(),
                    )?;
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
