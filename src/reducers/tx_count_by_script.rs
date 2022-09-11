// used by crfa in prod, c2, c3 key

use std::collections::HashSet;

use pallas::ledger::traverse::{Feature, MultiEraBlock, OutputRef};
use serde::Deserialize;

use crate::crosscut::epochs::block_epoch;

use crate::{crosscut, model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub aggr_by: Option<String>,
    pub addr_type: Option<String>,
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
            Some(aggr) if aggr == "EPOCH" => {
                let k = match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}.{}", prefix, address, epoch_no),
                    None => format!("{}.{}", def_key_prefix.to_string(), address),
                };

                return k;
            }
            _ => {
                let k = match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}", prefix, address),
                    None => format!("{}.{}", def_key_prefix.to_string(), address),
                };

                return k;
            }
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
            None => {
                log::warn!("UTxO:{} at index not found, index:{}", input.hash(), input.index());
                return Result::Ok(None);
            }
        };

        let is_script_address = utxo.address().map_or(false, |x| x.has_script());

        if !is_script_address {
            return Ok(None);
        }

        let address = utxo.address()
        .map(|x| {
            match &self.config.addr_type {
                Some(addr_typ) if addr_typ == "HEX" => x.to_hex(),
                _ => x.to_string()
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
                if tx.is_valid() {
                    let epoch_no = block_epoch(&self.chain, block);

                    let input_addresses: Vec<_> = tx
                        .inputs()
                        .iter()
                        .filter_map(|multi_era_input| {
                            let output_ref = multi_era_input.output_ref();

                            let maybe_input_address =
                                self.find_address_from_output_ref(ctx, &output_ref);

                            match maybe_input_address {
                                Ok(maybe_addr) => maybe_addr,
                                Err(x) => {
                                    log::error!(
                                        "Not found, tx_id:{}, index_at:{}, e:{}",
                                        output_ref.hash(),
                                        output_ref.index(),
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
                        .filter(|p| p.address().map_or(false, |a| a.has_script()))
                        .filter_map(|tx| tx.address().ok())
                        .filter(|a| a.has_script())
                        .map(|x| -> String {
                            match &self.config.addr_type {
                                Some(addr_typ) if addr_typ == "HEX" => x.to_hex(),
                                _ => x.to_string()
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
