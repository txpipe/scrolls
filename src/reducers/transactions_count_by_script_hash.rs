use pallas::ledger::addresses;
use pallas::ledger::traverse::{Feature, MultiEraBlock};
use serde::Deserialize;

use crate::crosscut::ChainWellKnownInfo;
use crate::model;

use pallas::ledger::primitives::ToHash;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    chain: ChainWellKnownInfo,
}

impl Reducer {
    fn send_count(
        &mut self,
        addr_hex: String,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, addr_hex),
            None => format!("{}", addr_hex),
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
                let maybe_plutus_scripts = &&tx
                    .as_alonzo()
                    .unwrap()
                    .transaction_witness_set
                    .plutus_script;

                if let Some(plutus_scripts) = maybe_plutus_scripts {
                    for plutus_script in plutus_scripts.iter() {
                        let hash = plutus_script.to_hash();

                        let addr = addresses::ShelleyAddress::new(
                            self.chain.address_network_id.into(),
                            addresses::ShelleyPaymentPart::Script(hash),
                            addresses::ShelleyDelegationPart::Null,
                        )
                        .to_hex();

                        self.send_count(addr, output)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, chain: &ChainWellKnownInfo) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            chain: chain.clone(),
        };

        super::Reducer::TransactionsCountByScriptHash(reducer)
    }
}
