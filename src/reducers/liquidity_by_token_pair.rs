use pallas::ledger::traverse::{Asset, MultiEraBlock, MultiEraOutput, MultiEraTx, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

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

fn contains_currency_symbol(currency_symbol: &String, assets: &Vec<Asset>) -> bool {
    assets.iter().any(|asset| {
        asset
            .policy_hex()
            .or(Some(String::new())) // in case ADA is part of the vector
            .unwrap()
            .as_str()
            .eq(currency_symbol)
    })
}

impl Reducer {
    fn process_consumed_txo(
        &mut self,
        ctx: &model::BlockContext,
        input: &OutputRef,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let utxo = ctx.find_utxo(input).apply_policy(&self.policy).or_panic()?;
        // Skip invalid or byron era outputs
        let utxo = match utxo {
            None => return Ok(()),
            Some(MultiEraOutput::Byron(_)) => return Ok(()),
            Some(x) => x,
        };

        // Skip any spending transaction that has no currency symbol of an identifiable liquidity source
        if !contains_currency_symbol(
            &self.config.pool_currency_symbol,
            utxo.non_ada_assets().as_ref(),
        ) {
            return Ok(());
        }

        Ok(())
    }

    fn process_produced_txo(
        &mut self,
        tx: &MultiEraTx,
        tx_output: &MultiEraOutput,
        output_idx: usize,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        // ToDo: Implementation
        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            for consumed in tx.consumes().iter().map(|i| i.output_ref()) {
                self.process_consumed_txo(&ctx, &consumed, output)?;
            }

            for (idx, produced) in tx.produces() {
                self.process_produced_txo(&tx, &produced, idx, output)?;
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::contains_currency_symbol;
    use pallas::ledger::{primitives::babbage::PolicyId, traverse::Asset};

    #[test]
    fn ada_currency_symbol() {
        let currency_symbol = "93744265ed9762d8fa52c4aacacc703aa8c81de9f6d1a59f2299235b";
        let mock_assets: Vec<Asset> = [
            Asset::NativeAsset(
                PolicyId::from_str(currency_symbol).ok().unwrap(),
                "Tkn1".to_string().as_bytes().to_vec(),
                1,
            ),
            Asset::NativeAsset(
                PolicyId::from_str(currency_symbol).ok().unwrap(),
                "Tkn2".to_string().as_bytes().to_vec(),
                1,
            ),
            Asset::NativeAsset(
                PolicyId::from_str("158fd94afa7ee07055ccdee0ba68637fe0e700d0e58e8d12eca5be46")
                    .ok()
                    .unwrap(),
                "Tkn3".to_string().as_bytes().to_vec(),
                1,
            ),
        ]
        .to_vec();
        assert_eq!(
            contains_currency_symbol(&currency_symbol.to_string(), &mock_assets),
            true
        );
        assert_eq!(
            contains_currency_symbol(&"".to_string(), &mock_assets),
            false
        );
        assert_eq!(
            contains_currency_symbol(&"123abc".to_string(), &mock_assets),
            false
        );
    }
}
