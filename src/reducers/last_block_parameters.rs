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

    pub fn current_epoch(
        &mut self,
        block: &MultiEraBlock,
        key: &str,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let epoch_no = block_epoch(&self.chain, block);

        let crdt = model::CRDTCommand::AnyWriteWins(format!("{}.{}", key, "epoch_no"), Value::BigInt(epoch_no as i128));

        output.send(gasket::messaging::Message::from(crdt))?;

        Result::Ok(())
    } 

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {

        let def_key_prefix = "current_block";

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}", prefix),
            None => format!("{}", def_key_prefix.to_string()),
        };

        self.current_epoch(block, &key, output)?;

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

        super::Reducer::LastBlockParameters(reducer)
    }
}
