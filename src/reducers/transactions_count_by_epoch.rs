use serde::Deserialize;

use crate::{crosscut, model};
use pallas::ledger::traverse::MultiEraBlock;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    chain: crosscut::ChainWellKnownInfo,
}

impl Reducer {
    fn increment_key(
        &mut self,
        epoch: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let prefix = match &self.config.key_prefix {
            Some(prefix) => prefix,
            None => "transactions_by_epoch",
        };

        let key = format!("{}.{}", prefix, epoch);

        let crdt = model::CRDTCommand::PNCounter(key, 1);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block(
        &mut self,
        block: &MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let epoch = crosscut::epochs::block_epoch(&self.chain, block);

        for _tx in block.txs() {
            // TODO apply filters using tx data
            self.increment_key(epoch, output)?;
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, chain: &crosscut::ChainWellKnownInfo) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            chain: chain.clone(),
        };

        super::Reducer::TransactionsCountByEpoch(reducer)
    }
}
