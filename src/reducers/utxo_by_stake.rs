use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{MultiEraBlock, MultiEraTx, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
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

        let stake_address = self.get_stake_from_utxo(&utxo);

        let stake_address = match stake_address {
            Some(x) => x,
            None => return Ok(()),
        };

        if let Some(stake_addresses) = &self.config.filter {
            if let Err(_) = stake_addresses.binary_search(&stake_address) {
                return Ok(());
            }
        }

        let crdt = model::CRDTCommand::set_remove(
            self.config.key_prefix.as_deref(),
            &stake_address,
            input.to_string(),
        );

        output.send(crdt.into())
    }

    fn process_produced_txo(
        &mut self,
        tx: &MultiEraTx,
        tx_output: &MultiEraOutput,
        output_idx: usize,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let tx_hash = tx.hash();
        let stake_address = self.get_stake_from_utxo(tx_output);

        let stake_address = match stake_address {
            Some(x) => x,
            None => return Ok(()),
        };

        if let Some(stake_addresses) = &self.config.filter {
            if let Err(_) = stake_addresses.binary_search(&stake_address) {
                return Ok(());
            }
        }

        let crdt = model::CRDTCommand::set_add(
            self.config.key_prefix.as_deref(),
            &stake_address,
            format!("{}#{}", tx_hash, output_idx),
        );

        output.send(crdt.into())
    }

    fn get_stake_from_utxo(&mut self, output: &MultiEraOutput) -> Option<String> {
        let stake_address = match output.address().unwrap() {
            pallas::ledger::addresses::Address::Shelley(shelley_addr) => {
                Some(shelley_addr.delegation().to_bech32().unwrap())
            }
            pallas::ledger::addresses::Address::Byron(_) => None,
            pallas::ledger::addresses::Address::Stake(_) => None,
        };

        stake_address
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

        super::Reducer::UtxoByStake(reducer)
    }
}
