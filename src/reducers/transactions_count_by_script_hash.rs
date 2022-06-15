use pallas::ledger::traverse::{Feature, MultiEraBlock};
use serde::Deserialize;

use crate::model;

use pallas::crypto::hash::Hash;

use pallas::ledger::primitives::ToHash;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
}

impl Reducer {
    fn send_count(
        &mut self,
        hash: Hash<28>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, hash.to_string()),
            None => format!("{}", hash.to_string()),
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
                let maybe_plutus_scripts = &&tx.as_alonzo().unwrap().transaction_witness_set.plutus_script;

                if let Some(plutus_scripts) = maybe_plutus_scripts {
                    for plutus_script in plutus_scripts.iter() {
                        let hash = plutus_script.to_hash();
        
                        self.send_count(hash, output)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let reducer = Reducer {
            config: self,
        };

        super::Reducer::TransactionsCountByScriptHash(reducer)
    }
}
