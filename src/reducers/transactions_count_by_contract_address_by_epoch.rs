use pallas::ledger::{
    primitives::alonzo,
    traverse::{Feature, MultiEraBlock},
};
use serde::Deserialize;

use crate::{
    crosscut::{self, EpochCalculator},
    model,
};

use core::slice::SlicePattern;
use std::collections::HashSet;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    address_hrp: String,
    shelley_known_slot: u64,
    shelley_epoch_length: u64,
}

impl Reducer {
    fn increment_for_addresses(
        &mut self,
        address: &str,
        slot: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let epoch_no = EpochCalculator::get_shelley_epoch_no_for_absolute_slot(
            self.shelley_known_slot,
            self.shelley_epoch_length,
            slot,
        );

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}.{}", prefix, address.to_string(), epoch_no),
            None => format!("{}.{}", address.to_string(), epoch_no),
        };

        let crdt = model::CRDTCommand::PNCounter(key, 1);
        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block(
        &mut self,
        block: &MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if block.era().has_feature(Feature::SmartContracts) {
            let slot = block.slot();

            for tx in block.txs() {
                let addresses: HashSet<_> = tx
                    .outputs()
                    .iter()
                    .filter_map(|tx| tx.as_alonzo())
                    .filter(|x| crosscut::addresses::is_smart_contract(x.address.as_slice()))
                    .filter_map(|x| x.to_bech32_address(&self.address_hrp).ok())
                    .collect();

                for address in addresses {
                    self.increment_for_addresses(address, slot, output)?;
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
            shelley_known_slot: chain.shelley_known_slot.clone() as u64,
            shelley_epoch_length: chain.shelley_epoch_length.clone() as u64,
        };

        super::Reducer::TransactionsCountByContractAddressByEpoch(reducer)
    }
}
