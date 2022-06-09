use pallas::ledger::primitives::alonzo;
use pallas::ledger::primitives::alonzo::{PoolKeyhash, StakeCredential};
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

    fn reduce_alonzo_compatible_tx(
        &mut self,
        slot: u64,
        tx: &alonzo::TransactionBody,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        tx.iter()
            .filter_map(|b| match b {
                alonzo::TransactionBodyComponent::Certificates(c) => Some(c),
                _ => None,
            })
            .flat_map(|c| c.iter())
            .filter_map(|c| match c {
                alonzo::Certificate::StakeDelegation(cred, pool) => Some((cred, pool)),
                _ => None,
            })
            .map(|(cred, pool)| self.send_key_write(cred, pool, slot, output))
            .collect()
    }

    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(block) => block
                .1
                .transaction_bodies
                .iter()
                .map(|tx| {
                    self.reduce_alonzo_compatible_tx(block.1.header.header_body.slot, tx, output)
                })
                .collect(),
        }
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let reducer = Reducer { config: self };
        super::Reducer::PoolByStake(reducer)
    }
}
