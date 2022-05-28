use pallas::ledger::primitives::alonzo;
use serde::Deserialize;

use crate::model;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
}

impl Reducer {
    fn increment_for_contract_address(
        &mut self,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => prefix.to_string(),
            None => "total_transactions_count_by_contract_addresses".to_string(),
        };

        let crdt = model::CRDTCommand::PNCounter(key, "1".to_string());
        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    fn reduce_alonzo_compatible_tx(
        &mut self,
        tx: &alonzo::TransactionBody,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let is_smart_contract_transaction = tx
            .iter()
            .filter_map(|b| match b {
                alonzo::TransactionBodyComponent::Outputs(o) => Some(o),
                _ => None,
            })
            .flat_map(|o| o.iter())
            .enumerate()
            .any(move |(_tx_idx, output)| {
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

                return is_smart_contract_address;
            });

        if is_smart_contract_transaction {
            return self.increment_for_contract_address(output);
        }

        return Ok(());
    }

    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(x) => {
                x.1.transaction_bodies
                    .iter()
                    .map(|tx| self.reduce_alonzo_compatible_tx(tx, output))
                    .collect()
            }
        }
    }
}

impl Config {
    pub fn plugin(self) -> super::Plugin {
        let reducer = Reducer { config: self };
        super::Plugin::TotalTransactionsCountByContractAddresses(reducer)
    }
}
