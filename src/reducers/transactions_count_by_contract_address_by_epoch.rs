use pallas::ledger::traverse::{Feature, MultiEraBlock, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

use std::collections::HashSet;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    chain: crosscut::ChainWellKnownInfo,
    policy: crosscut::policies::RuntimePolicy,
}

impl Reducer {
    fn increment_for_address(
        &mut self,
        address: &str,
        epoch_no: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}.{}", prefix, epoch_no, address.to_string()),
            None => format!("{}.{}", address.to_string(), epoch_no),
        };

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
            None => {
                log::error!("Output index not found, index:{}", input.tx_index());
                return Result::Ok(None);
            }
        };

        let is_script_address = utxo.to_address().map_or(false, |x| x.has_script());

        if !is_script_address {
            return Ok(None);
        }

        let address = utxo.to_address().and_then(|x| x.to_bech32()).ok();

        return match address {
            Some(addr) => Ok(Some(addr)),
            None => Ok(None)
        }
    }

    pub fn reduce_block(
        &mut self,
        block: &MultiEraBlock,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if block.era().has_feature(Feature::SmartContracts) {
            let epoch_no = crosscut::epochs::block_epoch(&self.chain, block);

            for tx in block.txs() {

                let input_addresses: Vec<_> = tx
                    .inputs()
                    .iter()
                    .filter_map(|multi_era_input| {
                
                        let output_ref = multi_era_input.output_ref().unwrap();

                        let maybe_input_address =
                            self.find_address_from_output_ref(ctx, &output_ref);

                        match maybe_input_address {
                            Ok(maybe_addr) => maybe_addr,
                            Err(x) => {
                                log::error!(
                                    "Not found, tx_id:{}, index_at:{}, e:{}",
                                    output_ref.tx_id(),
                                    output_ref.tx_index(),
                                    x
                                );
                                None
                            }
                        }
                    })
                    .collect();

                let output_addresses: Vec<_> = tx
                    .outputs()
                    .iter()
                    // allow only smart contract addresses 
                    .filter(|p| p.to_address().map_or(false, |a| a.has_script()))
                    .filter_map(|x| x.to_address().ok())
                    .filter_map(|x| x.to_bech32().ok())
                    .collect();

                let all_addresses = [&input_addresses[..], &output_addresses[..]].concat();

                let all_addresses_deduped: HashSet<String> =
                    HashSet::from_iter(all_addresses.iter().cloned());

                for address in all_addresses_deduped.iter() {
                    self.increment_for_address(address, epoch_no, output)?;
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        policy: &crosscut::policies::RuntimePolicy,
    ) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            chain: chain.clone(),
            policy: policy.clone(),
        };

        super::Reducer::TransactionsCountByContractAddressByEpoch(reducer)
    }
}
