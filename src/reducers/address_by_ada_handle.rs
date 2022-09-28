use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{Asset, MultiEraBlock};
use serde::Deserialize;

use crate::{model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
    pub policy_id_hex: String,
}

pub struct Reducer {
    config: Config,
}

impl Reducer {
    fn to_ada_handle(&self, asset: Asset) -> Option<String> {
        match asset.policy_hex() {
            Some(policy_id) if policy_id.eq(&self.config.policy_id_hex) => asset.ascii_name(),
            _ => None,
        }
    }

    pub fn process_txo(
        &self,
        txo: &MultiEraOutput,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let handles: Vec<_> = txo
            .non_ada_assets()
            .into_iter()
            .filter_map(|x| self.to_ada_handle(x))
            .collect();

        if handles.is_empty() {
            return Ok(());
        }

        let address = txo.address().map(|x| x.to_string()).or_panic()?;

        for handle in handles {
            log::debug!("ada handle match found: ${handle}=>{address}");

            let crdt = model::CRDTCommand::any_write_wins(
                self.config.key_prefix.as_deref(),
                handle,
                address.clone(),
            );

            output.send(crdt.into())?;
        }

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        _ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().iter() {
            for (_, txo) in tx.produces() {
                self.process_txo(&txo, output)?;
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let reducer = Reducer { config: self };

        super::Reducer::AddressByAdaHandle(reducer)
    }
}
