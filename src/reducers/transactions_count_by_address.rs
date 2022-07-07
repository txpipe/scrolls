use std::collections::HashSet;

use pallas::ledger::traverse::{Feature, MultiEraBlock, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
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

        let address = utxo.to_address().and_then(|x| x.to_bech32()).or_panic()?;

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
                    .filter_map(|tx| tx.to_address().ok())
                    .filter_map(|addr| addr.to_bech32().ok())
                    .collect();

                let all_addresses = [&input_addresses[..], &output_addresses[..]].concat();
                let all_addresses_deduped: HashSet<String> =
                    HashSet::from_iter(all_addresses.iter().cloned());

                for address in all_addresses_deduped.iter() {
                    self.increment_for_address(address, output)?;
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

        super::Reducer::TransactionsCountByAddress(reducer)
    }
}
