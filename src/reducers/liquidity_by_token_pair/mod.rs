use pallas::ledger::traverse::{MultiEraBlock, MultiEraOutput, MultiEraTx, OutputRef};
use serde::Deserialize;

pub mod minswap;
pub mod model;
pub mod utils;

use crate::{crosscut, prelude::*};

use self::utils::{contains_currency_symbol, filter_by_native_fungible};

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

impl Reducer {
    fn process_consumed_txo(
        &mut self,
        ctx: &crate::model::BlockContext,
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
        // let fungible_non_ada_assets = filter_by_native_fungible(utxo.non_ada_assets());

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
        ctx: &crate::model::BlockContext,
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
