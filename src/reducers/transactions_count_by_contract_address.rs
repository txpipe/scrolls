use std::collections::HashSet;

use crosscut::policies::*;
use gasket::error::AsWorkError;
use pallas::ledger::traverse::{Feature, MultiEraBlock, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub policy: Option<ReducerPolicy>,
}

pub struct Reducer {
    config: Config,
    address_hrp: String,
}

impl Reducer {
    fn increment_for_address(
        &mut self,
        address: &str,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, address.to_string()),
            None => format!("{}", address.to_string()),
        };

        let crdt = model::CRDTCommand::PNCounter(key, 1);
        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    fn find_address_from_output_ref(
        &mut self,
        ctx: &model::BlockContext,
        input: &OutputRef
    ) -> Result<Option<String>, gasket::error::Error> {
        let inbound_tx = ctx
        .find_ref_tx(input.tx_id())
        .apply_policy(&self.config.policy)
        .or_work_err()?;

        let inbound_tx = match inbound_tx {
            Some(x) => x,
            None => return Result::Ok(None),
        };

        let output_tx = inbound_tx
            .output_at(input.tx_index() as usize)
            .ok_or(crate::Error::ledger("output index not found in tx"))
            .apply_policy(&self.config.policy)
            .or_work_err()?;

        match output_tx {
            Some(x) => Result::Ok(Some(x.address(&self.address_hrp))),
            None => Result::Ok(None)
        }
    }

    pub fn reduce_block(
        &mut self,
        block: &MultiEraBlock,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if block.era().has_feature(Feature::SmartContracts) {
            for tx in block.txs() {

                let input_addresses: Vec<_> = tx
                .inputs()
                .iter()
                .filter_map(|multi_era_input| { 
                    let output_ref = multi_era_input.output_ref().unwrap();

                    let maybe_input_address = self.find_address_from_output_ref(ctx, &output_ref);

                    match maybe_input_address {
                        Ok(maybe_addr) => maybe_addr,
                        Err(_) => None 
                    }
                })
                .filter(|x| crosscut::addresses::is_smart_contract(x.as_bytes()))
                .collect();

                let output_addresses: Vec<_> = tx
                    .outputs()
                    .iter()
                    .filter_map(|tx| tx.as_alonzo())
                    .filter(|x| crosscut::addresses::is_smart_contract(x.address.as_slice()))
                    .filter_map(|x| x.to_bech32_address(&self.address_hrp).ok())
                    .collect();

                let all_addresses = [&input_addresses[..], &output_addresses[..]].concat();
                let all_addresses_deduped: HashSet<String> = HashSet::from_iter(all_addresses.iter().cloned());

                for address in all_addresses_deduped.iter() {
                    self.increment_for_address(address, output)?;
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, chain: &crosscut::ChainWellKnownInfo) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            address_hrp: chain.address_hrp.clone(),
        };

        super::Reducer::TransactionsCountByContractAddress(reducer)
    }
}
