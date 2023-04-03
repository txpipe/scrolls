use pallas::ledger::addresses::{Address, StakeAddress};
use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{MultiEraBlock, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<crosscut::filters::Predicate>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
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
    fn process_consumed_txo(
        &mut self,
        ctx: &model::BlockContext,
        input: &OutputRef,
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

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, stake_address),
            None => format!("{}.{}", "balance_by_stake_key".to_string(), stake_address),
        };

        let crdt = model::CRDTCommand::PNCounter(key, -1 * utxo.lovelace_amount() as i64);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    fn process_produced_txo(
        &mut self,
        tx_output: &MultiEraOutput,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let address = tx_output.address().or_panic()?;
        let stake_address = any_address_to_stake_bech32(address);

        let stake_address = match stake_address {
            Some(x) => x,
            None => return Ok(()),
        };
        
        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, stake_address),
            None => format!("{}.{}", "balance_by_stake_key".to_string(), stake_address),
        };

        let crdt = model::CRDTCommand::PNCounter(key, tx_output.lovelace_amount() as i64);

        output.send(gasket::messaging::Message::from(crdt))?;

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
                for consumed in tx.consumes().iter().map(|i| i.output_ref()) {
                    self.process_consumed_txo(&ctx, &consumed, output)?;
                }

                for (_, produced) in tx.produces() {
                    self.process_produced_txo(&produced, output)?;
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

        super::Reducer::BalanceByStakeKey(reducer)
    }
}
