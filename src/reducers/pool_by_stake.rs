use pallas::ledger::primitives::alonzo;
use pallas::ledger::primitives::alonzo::{PoolKeyhash, StakeCredential};
use pallas::ledger::traverse::MultiEraBlock;
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
    fn send_key_write(
        &mut self,
        cred: &StakeCredential,
        pool: &PoolKeyhash,
        slot: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let hash = match cred {
            StakeCredential::AddrKeyhash(x) => x.to_string(),
            StakeCredential::Scripthash(x) => x.to_string(),
        };

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, hash),
            None => hash.to_string(),
        };

        let value = pool.to_string();

        let crdt = model::CRDTCommand::LastWriteWins(key, value, slot);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let slot = block.slot();

        for tx in block.txs() {
            for cert in tx.certs() {
                if let Some(cert) = cert.as_alonzo() {
                    if let alonzo::Certificate::StakeDelegation(cred, pool) = cert {
                        self.send_key_write(cred, pool, slot, output)?;
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
        super::Reducer::PoolByStake(reducer)
    }
}
