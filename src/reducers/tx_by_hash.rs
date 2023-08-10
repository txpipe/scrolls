use pallas::ledger::traverse::{MultiEraBlock, MultiEraTx};
use serde::Deserialize;
use serde_json::json;

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
    time: crosscut::time::NaiveProvider,
}

impl Reducer {
    fn send(
        &mut self,
        block: &MultiEraBlock,
        tx: &MultiEraTx,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key_prefix = self.config.key_prefix.as_deref();
        let crdt = match self.config.projection.unwrap_or_default() {
            Projection::Cbor => {
                let cbor = tx.encode();
                model::CRDTCommand::any_write_wins(key_prefix, tx.hash(), cbor)
            }
            Projection::Json => {
                let cbor = tx.encode();
                let slot = block.slot();
                let ts = self.time.slot_to_wallclock(slot);
                let json = json!({ "cbor": hex::encode(cbor), "slot": slot, "time": ts});
                model::CRDTCommand::any_write_wins(key_prefix, tx.hash(), json.to_string())
            }
        };

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        rollback: bool,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if rollback {
            return Ok(());
        }

        for tx in &block.txs() {
            if filter_matches!(self, block, &tx, ctx) {
                self.send(block, tx, output)?;
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        policy: &crosscut::policies::RuntimePolicy,
    ) -> super::Reducer {
        let worker = Reducer {
            config: self,
            policy: policy.clone(),
            time: crosscut::time::NaiveProvider::new(chain.clone()),
        };
        super::Reducer::TxByHash(worker)
    }
}
