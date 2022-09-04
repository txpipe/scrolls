use pallas::ledger::traverse::{MultiEraBlock, MultiEraTx};
use serde::Deserialize;

use crate::prelude::*;
use crate::{crosscut, model};

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
    fn send(
        &mut self,
        tx: &MultiEraTx,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let cbor = tx
            .encode()
            .map_err(crate::Error::cbor)
            .apply_policy(&self.policy)
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
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in &block.txs() {
            if filter_matches!(self, block, &tx, ctx) {
                self.send(tx, output)?;
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, policy: &crosscut::policies::RuntimePolicy) -> super::Reducer {
        let worker = Reducer {
            config: self,
            policy: policy.clone()
        };
        super::Reducer::TxByHash(worker)
    }
}
