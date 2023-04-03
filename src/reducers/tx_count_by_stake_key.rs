use pallas::ledger::addresses::{Address, StakeAddress};
use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{MultiEraBlock, OutputRef};
use serde::Deserialize;
use std::collections::HashSet;

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
    fn process_inbound_txo(
        &mut self,
        ctx: &model::BlockContext,
        input: &OutputRef,
        seen: &mut HashSet<String>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let utxo = ctx.find_utxo(input).apply_policy(&self.policy).or_panic()?;

        let utxo = match utxo {
            Some(x) => x,
            None => return Ok(())
        };

        let address = utxo.address().or_panic()?;
        let stake_address = any_address_to_stake_bech32(address);

        let stake_address = match stake_address {
            Some(x) => x,
            None => return Ok(()),
        };
        
        if seen.insert(stake_address.clone()) {
            let key = match &self.config.key_prefix {
                Some(prefix) => format!("{}.{}", prefix, stake_address),
                None => format!("{}.{}", "tx_count_by_stake_key".to_string(), stake_address),
            };
    
            let crdt = model::CRDTCommand::PNCounter(key, 1);
    
            output.send(gasket::messaging::Message::from(crdt))?;
        }

        Ok(())
    }

    fn process_outbound_txo(
        &mut self,
        tx_output: &MultiEraOutput,
        seen: &mut HashSet<String>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let address = tx_output.address().or_panic()?;
        let stake_address = any_address_to_stake_bech32(address);

        let stake_address = match stake_address {
            Some(x) => x,
            None => return Ok(()),
        };
        
        if seen.insert(stake_address.clone()) {
            let key = match &self.config.key_prefix {
                Some(prefix) => format!("{}.{}", prefix, stake_address),
                None => format!("{}.{}", "tx_count_by_stake_key".to_string(), stake_address),
            };
    
            let crdt = model::CRDTCommand::PNCounter(key, 1);
    
            output.send(gasket::messaging::Message::from(crdt))?;
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
                let mut seen = HashSet::new();
                
                for input in tx.inputs().iter().map(|i| i.output_ref()) {
                    self.process_inbound_txo(&ctx, &input, &mut seen, output)?;
                }

                for (_idx, tx_output) in tx.outputs().iter().enumerate() {
                    self.process_outbound_txo(tx_output, &mut seen, output)?;
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

        super::Reducer::TxCountByStakeKey(reducer)
    }
}
