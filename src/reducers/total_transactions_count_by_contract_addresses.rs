use pallas::ledger::traverse::{Feature, MultiEraBlock};
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
}

impl Reducer {
    fn increment_for_address(
        &mut self,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => prefix.to_string(),
            None => "total_transactions_count_by_contract_addresses".to_string(),
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
            for tx in block.txs() {
                for tx_out in tx.outputs().iter().filter_map(|x| x.as_alonzo()) {
                    if crosscut::addresses::is_smart_contract(tx_out.address.as_slice()) {
                        self.increment_for_address(output)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let reducer = Reducer { config: self };
        super::Reducer::TotalTransactionsCountByContractAddresses(reducer)
    }
}
