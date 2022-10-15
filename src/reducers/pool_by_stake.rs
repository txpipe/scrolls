use pallas::ledger::primitives::alonzo;
use pallas::ledger::primitives::alonzo::{PoolKeyhash, StakeCredential};
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<crosscut::filters::Predicate>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
}

impl Reducer {
    fn send_key_write(
        &mut self,
        cred: &StakeCredential,
        pool: &PoolKeyhash,
        slot: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match cred {
            StakeCredential::AddrKeyhash(x) => x.to_string(),
            StakeCredential::Scripthash(x) => x.to_string(),
        };

        let value = pool.to_string();

        let crdt = model::CRDTCommand::last_write_wins(
            self.config.key_prefix.as_deref(),
            &key,
            value,
            slot,
        );

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let slot = block.slot();

        for tx in block.txs() {
            if filter_matches!(self, block, &tx, ctx) {
                if tx.is_valid() {
                    for cert in tx.certs() {
                        if let Some(cert) = cert.as_alonzo() {
                            if let alonzo::Certificate::StakeDelegation(cred, pool) = cert {
                                self.send_key_write(cred, pool, slot, output)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, 
        policy: &crosscut::policies::RuntimePolicy) -> super::Reducer {
        let reducer = Reducer { 
            config: self,
            policy: policy.clone()
         };
        super::Reducer::PoolByStake(reducer)
    }
}
