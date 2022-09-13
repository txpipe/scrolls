use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::crosscut::epochs::block_epoch;
use crate::model::Value;
use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    chain: crosscut::ChainWellKnownInfo,
}

impl Reducer {

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let epoch_no = block_epoch(&self.chain, block);

        let def_key_prefix = "current_epoch";

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, epoch_no),
            None => format!("{}.{}", def_key_prefix.to_string(), epoch_no),
        };

        let crdt = model::CRDTCommand::AnyWriteWins(key, Value::String(epoch_no.to_string()));

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }
}

impl Config {
    pub fn plugin(self,
         chain: &crosscut::ChainWellKnownInfo
         ) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            chain: chain.clone(),
        };

        super::Reducer::CurrentEpoch(reducer)
    }
}
