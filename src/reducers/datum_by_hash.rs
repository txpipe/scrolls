use pallas::ledger::primitives::babbage::PlutusData;
use pallas::ledger::primitives::{Fragment, ToCanonicalJson};
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize, Default)]
pub enum Projection {
    #[default]
    Cbor,
    Json,
}

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
    pub projection: Option<Projection>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
}

impl Reducer {
    fn process_datum(
        &mut self,
        slot: u64,
        datum: &PlutusData,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let datum_hash = datum.to_hash();

        let crdt = match self.config.projection.unwrap_or_default() {
            Projection::Cbor => model::CRDTCommand::any_write_wins(
                self.config.key_prefix.as_deref(),
                &datum_hash,
                datum.encode_fragment().unwrap(),
            ),
            Projection::Json => model::CRDTCommand::any_write_wins(
                self.config.key_prefix.as_deref(),
                &datum_hash,
                datum.to_json(),
            ),
        };

        output.send(crdt.into())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let slot = block.slot();

        for tx in block.txs().into_iter() {
            if let Some(plutus_data) = tx.witnesses().plutus_data() {
                for datum in plutus_data {
                    self.process_datum(slot, datum, output);
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, policy: &crosscut::policies::RuntimePolicy) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            policy: policy.clone(),
        };

        super::Reducer::DatumByHash(reducer)
    }
}
