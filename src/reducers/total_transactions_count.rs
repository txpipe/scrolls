use pallas::ledger::primitives::byron;
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
    fn increment_key(
        &mut self,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => prefix.to_string(),
            None => "total_transactions_count".to_string(),
        };

        let crdt = model::CRDTCommand::PNCounter(key, 1.to_string());

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(byron::Block::MainBlock(x)) => x
                .body
                .tx_payload
                .iter()
                .map(|_tx| self.increment_key(output))
                .collect(),

            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(block) => block
                .1
                .transaction_bodies
                .iter()
                .map(|_tx| self.increment_key(output))
                .collect(),
        }
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let reducer = Reducer { config: self };
        super::Reducer::TotalTransactionsCount(reducer)
    }
}
