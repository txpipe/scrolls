use pallas::ledger::traverse::{MultiEraBlock, MultiEraTx};
use serde::Deserialize;

use crate::prelude::*;
use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub policy: crosscut::policies::RuntimePolicy,
}

pub struct Reducer {
    config: Config,
}

impl Reducer {
    fn send(
        &mut self,
        tx: &MultiEraTx,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let cbor = tx
            .encode()
            .map_err(crate::Error::cbor)
            .apply_policy(&self.config.policy)
            .or_panic()?;

        let value = match cbor {
            Some(x) => x,
            None => return Ok(()),
        };

        let crdt =
            model::CRDTCommand::any_write_wins(self.config.key_prefix.as_deref(), tx.hash(), value);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in &block.txs() {
            self.send(tx, output)?;
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let worker = Reducer { config: self };
        super::Reducer::TxByHash(worker)
    }
}
