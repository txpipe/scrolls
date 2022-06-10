use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::{byron, ToHash};
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
    fn send_set_add(
        &mut self,
        tx_hash: Hash<32>,
        block_slot: u64,
        block_hash: Hash<32>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, tx_hash),
            None => format!("{}", tx_hash),
        };

        let member = format!("{},{}", block_slot, block_hash);
        let crdt = model::CRDTCommand::GrowOnlySetAdd(key, member);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(byron::Block::MainBlock(x)) => {
                let hash = x.header.to_hash();
                let slot = x.header.consensus_data.0.to_abs_slot();

                x.body
                    .tx_payload
                    .iter()
                    .map(|tx| tx.transaction.to_hash())
                    .map(|tx| self.send_set_add(tx, slot, hash, output))
                    .collect()
            }
            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(x) => {
                let slot = x.1.header.header_body.slot;
                let hash = x.1.header.header_body.block_body_hash;

                x.1.transaction_bodies
                    .iter()
                    .map(|tx| tx.to_hash())
                    .map(|tx| self.send_set_add(tx, slot, hash, output))
                    .collect()
            }
        }
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let worker = Reducer { config: self };
        super::Reducer::PointByTx(worker)
    }
}
