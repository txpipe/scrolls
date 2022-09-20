use pallas::ledger::addresses::Address;

use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{MultiEraBlock, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

use crate::crosscut::epochs::block_epoch;

#[derive(Deserialize, Copy, Clone)]
pub enum AggrType {
    Epoch,
}

#[derive(Deserialize, Copy, Clone)]
pub enum AddressType {
    Payment, Staking
}

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<crosscut::filters::Predicate>,
    pub aggr_by: Option<AggrType>,
    pub addr_type: AddressType,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    chain: crosscut::ChainWellKnownInfo,
}

impl Reducer {
    fn config_key(&self, address: &String, epoch_no: u64) -> String {
        let def_key_prefix = "unique_addresses_by_script";

        match &self.config.aggr_by {
            Some(aggr_type) => {
                match aggr_type {
                    AggrType::Epoch => {
                        let k = match &self.config.key_prefix {
                            Some(prefix) => format!("{}.{}.{}", prefix, address, epoch_no),
                            None => format!("{}.{}", def_key_prefix.to_string(), address),
                        };
        
                        return k;
                    }
                }
            },
            None => {
                let k = match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}", prefix, address),
                    None => format!("{}.{}", def_key_prefix.to_string(), address),
                };

                return k;
            },
        };
    }

    fn process_consumed_txo(
        &mut self,
        ctx: &model::BlockContext,
        input: &OutputRef,
        epoch_no: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let utxo = ctx.find_utxo(input).apply_policy(&self.policy).or_panic()?;

        let utxo = match utxo {
            Some(x) => x,
            None => return Ok(()),
        };

        if let Some(addr) = utxo.address().ok() {
            if !addr.has_script() {
                return Ok(());
            }

            let maybe_addr = match addr {
                Address::Shelley(shelley_addr) => {
                    match &self.config.addr_type {
                        AddressType::Staking => shelley_addr.payment().to_bech32().ok(),
                        AddressType::Payment => shelley_addr.delegation().to_bech32().ok()
                    }
                },
                _ => None,
            };

            if let Some(addr) =  &maybe_addr {
                let key = self.config_key(addr, epoch_no);
                let crdt = model::CRDTCommand::GrowOnlySetAdd(key, addr.to_string());

                output.send(gasket::messaging::Message::from(crdt))?;
            }

        }

        Ok(())
    }

    fn process_produced_txo(
        &mut self,
        tx_output: &MultiEraOutput,
        epoch_no: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {

        if let Some(addr) = tx_output.address().ok() {
            if !addr.has_script() {
                return Ok(());
            }

            let maybe_addr = match addr {
                Address::Shelley(shelley_addr) => {
                    match &self.config.addr_type {
                        AddressType::Staking => shelley_addr.payment().to_bech32().ok(),
                        AddressType::Payment => shelley_addr.delegation().to_bech32().ok()
                    }
                },
                _ => None,
            };

            if let Some(addr) =  &maybe_addr {
                let key = self.config_key(addr, epoch_no);
                let crdt = model::CRDTCommand::GrowOnlySetAdd(key, addr.to_string());

                output.send(gasket::messaging::Message::from(crdt))?;
            }

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
                let epoch_no = block_epoch(&self.chain, block);

                for consumed in tx.consumes().iter().map(|i| i.output_ref()) {
                    self.process_consumed_txo(&ctx, &consumed, epoch_no, output)?;
                }

                for produced in tx.produces().iter() {
                    self.process_produced_txo(produced, epoch_no, output)?;
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self,
         chain: &crosscut::ChainWellKnownInfo,
         policy: &crosscut::policies::RuntimePolicy,
        ) -> super::Reducer {

        let reducer = Reducer {
            config: self,
            chain: chain.clone(),
            policy: policy.clone(),
        };

        super::Reducer::UniqueAddressesByScript(reducer)
    }
}
