use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{Feature, MultiEraBlock, OutputRef};
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
    fn process_inbound_txo(
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

        let is_script_address = utxo.to_address().map_or(false, |x| x.has_script());

        if !is_script_address {
            return Ok(());
        }

        let address = utxo.to_address().and_then(|x| x.to_bech32()).ok();

        match address {
            Some(addr) => {

                if let Some(addresses) = &self.config.filter {
                    if let Err(_) = addresses.binary_search(&addr.to_string()) {
                        return Ok(());
                    }
                }
        
                let key = match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}", prefix, addr),
                    None => format!("{}.{}", "balance_by_contract_address".to_string(), addr),
                };
        
                let crdt = model::CRDTCommand::PNCounter(key, utxo.ada_amount() as i64);
        
                output.send(gasket::messaging::Message::from(crdt))?;
            },
            None => return Ok(()),
        }

        Ok(())
    }

    fn process_outbound_txo(
        &mut self,
        tx_output: &MultiEraOutput,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {

        let address = tx_output
            .to_address()
            .and_then(|x| x.to_bech32())
            .ok();

        match address {
            Some(addr) => {
                let is_script_address = tx_output.to_address().map_or(false, |x| x.has_script());

                if !is_script_address {
                    return Ok(());
                }
        
                if let Some(addresses) = &self.config.filter {
                    if let Err(_) = addresses.binary_search(&addr.to_string()) {
                        return Ok(());
                    }
                }

                let key = match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}", prefix, addr),
                    None => format!("{}.{}", "balance_by_contract_address".to_string(), addr),
                };
        
                let crdt = model::CRDTCommand::PNCounter(key, (-1) * tx_output.ada_amount() as i64);
        
                output.send(gasket::messaging::Message::from(crdt))?;
            }
            None => return Ok(()),
        }

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if block.era().has_feature(Feature::SmartContracts) {
            for tx in block.txs().into_iter() {
                for input in tx.inputs().iter().filter_map(|i| i.output_ref()) {
                    self.process_inbound_txo(&ctx, &input, output)?;
                }
    
                for (_idx, tx_output) in tx.outputs().iter().enumerate() {
                    self.process_outbound_txo(tx_output, output)?;
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

        super::Reducer::BalanceByContractAddress(reducer)
    }
}
