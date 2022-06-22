use pallas::ledger::traverse::{Feature, MultiEraBlock};
use serde::Deserialize;

use crate::{crosscut, model};

use std::collections::HashSet;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    address_hrp: String,
    chain: crosscut::ChainWellKnownInfo,
}

impl Reducer {
    fn increment_for_address(
        &mut self,
        address: &str,
        epoch: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}.{}", prefix, address.to_string(), epoch),
            None => format!("{}.{}", address.to_string(), epoch),
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
            let epoch = crosscut::epochs::block_epoch(&self.chain, block);

            for tx in block.txs() {
                let addresses: HashSet<_> = tx
                    .outputs()
                    .iter()
                    .filter_map(|tx| tx.as_alonzo())
                    .filter(|x| crosscut::addresses::is_smart_contract(x.address.as_slice()))
                    .filter_map(|x| x.to_bech32_address(&self.address_hrp).ok())
                    .collect();

                for address in addresses.iter() {
                    self.increment_for_address(address, epoch, output)?;
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
            chain: chain.clone(),
        };

        super::Reducer::TransactionsCountByAddressByEpoch(reducer)
    }
}
