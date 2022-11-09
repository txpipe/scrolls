use pallas::codec::utils::KeepRaw;
use pallas::ledger::primitives::babbage::{DatumOption, PlutusData};
use pallas::ledger::primitives::{Fragment, ToCanonicalJson};
use pallas::ledger::traverse::{MultiEraBlock, OriginalHash};
use serde::Deserialize;

use crate::model;

#[derive(Deserialize, Default, Copy, Clone)]
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
}

impl Reducer {
    fn process_datum(
        &mut self,
        datum: &KeepRaw<PlutusData>,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let datum_hash = datum.original_hash();

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
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            for datum in tx.plutus_data() {
                self.process_datum(datum, output)?;
            }

            for tx_out in tx.outputs() {
                if let Some(DatumOption::Data(datum)) = tx_out.datum().clone() {
                    self.process_datum(datum.deref(), output);
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let reducer = Reducer { config: self };

        super::Reducer::DatumByHash(reducer)
    }
}
