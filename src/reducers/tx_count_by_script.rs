// used by crfa in prod, c2, c3 key

use std::collections::HashSet;

use pallas::ledger::traverse::{Feature, MultiEraBlock, OutputRef};
use serde::Deserialize;

use crate::crosscut::epochs::block_epoch;

use crate::{crosscut, model, prelude::*};

#[derive(Deserialize, Copy, Clone, PartialEq)]
pub enum AggrType {
    Epoch,
}

#[derive(Deserialize, Copy, Clone, PartialEq)]
pub enum AddrType {
    Hex,
}

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub aggr_by: Option<AggrType>,
    pub key_addr_type: Option<AddrType>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    chain: crosscut::ChainWellKnownInfo,
}

impl Reducer {

    fn config_key(&self, address: String, epoch_no: u64) -> String {
        let def_key_prefix = "trx_count";

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

    fn increment_for_address(
        &mut self,
        address: &str,
        output: &mut super::OutputPort,
        epoch_no: u64,
    ) -> Result<(), gasket::error::Error> {
        let key = self.config_key(address.to_string(), epoch_no);

        let crdt = model::CRDTCommand::PNCounter(key, 1);
        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    fn find_address_from_output_ref(
        &mut self,
        ctx: &model::BlockContext,
        input: &OutputRef,
    ) -> Result<Option<String>, gasket::error::Error> {
        let utxo = ctx.find_utxo(input).apply_policy(&self.policy).or_panic()?;

        let utxo = match utxo {
            Some(x) => x,
            None => return Result::Ok(None)
        };

        let is_script_address = utxo.address().map_or(false, |addr| addr.has_script());

        if !is_script_address {
            return Ok(None);
        }

        let address = utxo.address()
        .map(|addr| {
            match &self.config.key_addr_type {
                Some(addr_typ) if matches!(addr_typ, AddrType::Hex) => addr.to_hex(),
                _ => addr.to_string()
            }
        })
        .or_panic()?;

        Ok(Some(address))
    }

    pub fn reduce_block(
        &mut self,
        block: &MultiEraBlock,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if block.era().has_feature(Feature::SmartContracts) {
            for tx in block.txs() {
                let epoch_no = block_epoch(&self.chain, block);

                let input_addresses: Vec<_> = tx
                    .consumes()
                    .iter()
                    .filter_map(|mei| {
                        let output_ref = mei.output_ref();

                        let maybe_input_address =
                            self.find_address_from_output_ref(ctx, &output_ref);

                        match maybe_input_address {
                            Ok(maybe_addr) => maybe_addr,
                            Err(_) => None
                        }
                    })
                    .collect();

                let output_addresses: Vec<_> = tx
                    .produces()
                    .iter()
                    .filter(|(_, meo)| meo.address().map_or(false, |a| a.has_script()))
                    .filter_map(|(_, meo)| meo.address().ok())
                    .filter(|addr| addr.has_script())
                    .map(|addr| -> String {
                        match &self.config.key_addr_type {
                            Some(addr_typ) if matches!(addr_typ, AddrType::Hex) => addr.to_hex(),
                            _ => addr.to_string()
                        }
                    })
                    .collect();

                let all_addresses = [&input_addresses[..], &output_addresses[..]].concat();
                let all_addresses_deduped: HashSet<String> =
                    HashSet::from_iter(all_addresses.iter().cloned());

                for address in all_addresses_deduped.iter() {
                    self.increment_for_address(address, output, epoch_no)?;
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, 
        chain: &crosscut::ChainWellKnownInfo,
        policy: &crosscut::policies::RuntimePolicy) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            policy: policy.clone(),
            chain: chain.clone(),
        };

        super::Reducer::TxCountByScript(reducer)
    }
}
