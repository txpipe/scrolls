use pallas::ledger::primitives::alonzo;
use serde::Deserialize;

use crate::{
    crosscut::{self, EpochCalculator},
    model,
};

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
        contract_addresses: &std::collections::HashSet<String>,
        slot: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let epoch_no = EpochCalculator::get_shelley_epoch_no_for_absolute_slot(
            self.shelley_known_slot,
            self.shelley_epoch_length,
            slot,
        );

        for contract_address in contract_addresses {
            let key = match &self.config.key_prefix {
                Some(prefix) => format!("{}.{}.{}", prefix, contract_address.to_string(), epoch_no),
                None => format!("{}.{}", contract_address.to_string(), epoch_no),
            };

            let crdt = model::CRDTCommand::PNCounter(key, "1".to_string());
            output.send(gasket::messaging::Message::from(crdt))?;
        }

        Ok(())
    }

    fn reduce_alonzo_compatible_tx(
        &mut self,
        tx: &alonzo::TransactionBody,
        slot: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let hrp_addr = &self.address_hrp.clone();

        let addresses: Vec<Option<String>> = tx
            .iter()
            .filter_map(|b| match b {
                alonzo::TransactionBodyComponent::Outputs(o) => Some(o),
                _ => None,
            })
            .flat_map(|o| o.iter())
            .enumerate()
            .map(move |(_tx_idx, output)| {
                let address = output.to_bech32_address(hrp_addr).unwrap();

                fn get_bit_at(input: u8, n: u8) -> bool {
                    if n < 32 {
                        input & (1 << n) != 0
                    } else {
                        false
                    }
                }

                // first byte of address is header
                let first_byte_of_address = output.address.as_slice()[0];
                // https://github.com/input-output-hk/cardano-ledger/blob/master/eras/alonzo/test-suite/cddl-files/alonzo.cddl#L135
                let is_smart_contract_address = get_bit_at(first_byte_of_address, 4);

                if is_smart_contract_address {
                    return Some(address);
                }

                return None::<String>;
            })
            .collect();

        if addresses.len() == 0 {
            return Result::Ok(());
        }

        let currated_addresses: Vec<String> = addresses
            .into_iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        let deduped_addresses: HashSet<String> = HashSet::from_iter(currated_addresses);

        return self.increment_for_addresses(&deduped_addresses, slot, output);
    }

    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(x) => {
                let slot = x.1.header.header_body.slot;

                x.1.transaction_bodies
                    .iter()
                    .map(|tx| self.reduce_alonzo_compatible_tx(tx, slot, output))
                    .collect()
            }
        }
    }
}

impl Config {
    pub fn plugin(self, chain: &crosscut::ChainWellKnownInfo) -> super::Plugin {
        let reducer = Reducer {
            config: self,
            address_hrp: chain.address_hrp.clone(),
            shelley_known_slot: chain.shelley_known_slot.clone() as u64,
            shelley_epoch_length: chain.shelley_epoch_length.clone() as u64,
        };

        super::Plugin::TransactionsCountByContractAddressByEpoch(reducer)
    }
}
