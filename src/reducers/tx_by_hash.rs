use pallas::ledger::traverse::{MultiEraBlock, MultiEraTx};
use serde::Deserialize;

use crate::prelude::*;
use crate::{crosscut, model};

#[derive(Deserialize, Copy, Clone)]
pub enum Projection {
    Cbor,
    Json,
}

impl Default for Projection {
    fn default() -> Self {
        Self::Cbor
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<crosscut::filters::Predicate>,
    pub projection: Option<Projection>,
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
        let key_prefix = self.config.key_prefix.as_deref();
        let crdt = match self.config.projection.unwrap_or_default() {
            Projection::Cbor => {
                let cbor = tx.encode();
                model::CRDTCommand::any_write_wins(key_prefix, tx.hash(), cbor)
            }
            Projection::Json => todo!(),
        };

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
            policy: policy.clone(),
        };
        super::Reducer::TxByHash(worker)
    }
}
