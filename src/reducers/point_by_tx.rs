use pallas::crypto::hash::Hash;
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
        block: &MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let block_hash = block.hash();
        let block_slot = block.slot();

        block
            .tx_iter()
            .map(|tx| self.send_set_add(tx.hash(), block_slot, block_hash, output))
            .collect()
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let worker = Reducer { config: self };
        super::Reducer::PointByTx(worker)
    }
}
