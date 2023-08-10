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

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        rollback: bool,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if rollback {
            return Ok(());
        }

        let block_hash = block.hash();
        let block_slot = block.slot();

        for tx in &block.txs() {
            self.send_set_add(tx.hash(), block_slot, block_hash, output)?;
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let worker = Reducer { config: self };
        super::Reducer::PointByTx(worker)
    }
}
