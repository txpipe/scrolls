use pallas::ledger::addresses::{Address, ShelleyDelegationPart};

use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

use crate::crosscut::epochs::block_epoch;

#[derive(Deserialize, Copy, Clone, PartialEq)]
pub enum AddrType {
    Hex,
}

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
    pub key_addr_type: Option<AddrType>,
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
            Some(aggr_type) if matches!(aggr_type, AggrType::Epoch) => {
                return match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}.{}", prefix, address, epoch_no),
                    None => format!("{}.{}", def_key_prefix.to_string(), address),
                };
            },
            _ => {
                return match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}", prefix, address),
                    None => format!("{}.{}", def_key_prefix.to_string(), address),
                };
            },
        };
    }

    fn process_user_address_given_contract_address(
        &mut self,
        contract_address: &Address,
        user_address: &Address,
        epoch_no: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {

        let maybe_addr = match user_address {
            Address::Shelley(shelley_addr) => {
                match &self.config.addr_type {
                    AddressType::Staking => {
                        let delegation_part = shelley_addr.delegation();

                        match delegation_part {
                            ShelleyDelegationPart::Key(_) | ShelleyDelegationPart::Script(_) => {
                                match &self.config.key_addr_type {
                                    Some(addr_typ) if matches!(addr_typ, AddrType::Hex) => Some(delegation_part.to_hex()),
                                    _ => delegation_part.to_bech32().ok()
                                }
                            },
                            _ => None,
                        }
                    },
                    AddressType::Payment => {
                        match &self.config.key_addr_type {
                            Some(addr_typ) if matches!(addr_typ, AddrType::Hex) => Some(shelley_addr.to_hex()),
                            _ => shelley_addr.to_bech32().ok()
                        }
                    },
                }
            },
            _ => None,
        };

        if let Some(addr) = &maybe_addr {

            match &self.config.key_addr_type {
                Some(addr_typ) if matches!(addr_typ, AddrType::Hex) => {
                    let key = self.config_key(&contract_address.to_hex(), epoch_no);
    
                    let crdt = model::CRDTCommand::GrowOnlySetAdd(key, addr.to_string());
        
                    output.send(gasket::messaging::Message::from(crdt))?;
                }
                _ => {
                    if let Some(contr_addr) = contract_address.to_bech32().ok() {
                        let key = self.config_key(&contr_addr, epoch_no);
        
                        let crdt = model::CRDTCommand::GrowOnlySetAdd(key, addr.to_string());
            
                        output.send(gasket::messaging::Message::from(crdt))?;
                    }
                }
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

                let enriched_inputs: Vec<MultiEraOutput> = tx.consumes().iter()
                .flat_map(|mei| ctx.find_utxo(&mei.output_ref()).apply_policy(&self.policy).or_panic().ok())
                .filter_map(|maybe_multi_era_output| maybe_multi_era_output)
                .collect();

                let inputs_have_script = enriched_inputs.iter().find(|meo| {
                    match meo.address().ok() {
                        Some(addr) => addr.has_script(),
                        None => false
                    }
                });

                let enriched_outputs: Vec<(usize, MultiEraOutput)> = tx.produces();

                let outputs_have_script = enriched_outputs.iter().find(|(_, meo)| {
                    match meo.address().ok() {
                        Some(addr) => addr.has_script(),
                        None => false
                    }
                });

                if let Some(meo) = inputs_have_script {

                    if let Some(contract_address) = &meo.address().ok() {

                        for meo in &enriched_inputs {
                            match meo.address().ok() {
                                Some(user_address) if !user_address.has_script() => self.process_user_address_given_contract_address(&contract_address, &user_address, epoch_no, output)?,
                                _ => (),
                            }
                        }

                        for (_, meo) in &enriched_outputs {
                            match meo.address().ok() {
                                Some(user_address) if !user_address.has_script() => self.process_user_address_given_contract_address(&contract_address, &user_address, epoch_no, output)?,
                                _ => (),
                            }
                        }

                    }
                }

                if let Some((_, meo)) = outputs_have_script {

                    if let Some(contract_address) = &meo.address().ok() {

                        for meo in &enriched_inputs {
                            match meo.address().ok() {
                                Some(user_address) if !user_address.has_script() => self.process_user_address_given_contract_address(&contract_address, &user_address, epoch_no, output)?,
                                _ => (),
                            }
                        }

                        for (_, meo) in &enriched_outputs {
                            match meo.address().ok() {
                                Some(user_address) if !user_address.has_script() => self.process_user_address_given_contract_address(&contract_address, &user_address, epoch_no, output)?,
                                _ => (),
                            }
                        }

                    }
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
