use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{Asset, MultiEraBlock};
use serde::Deserialize;

use crate::{model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
    pub policy_id_hex: String,
    // bool convert to ascii, default true
    pub convert_to_ascii: Option<bool>,
}

pub struct Reducer {
    config: Config,
    convert_to_ascii: bool,
}

impl Reducer {
    fn to_string_output(&self, asset: Asset) -> Option<String> {
        match asset.policy_hex() {
            Some(policy_id) if policy_id.eq(&self.config.policy_id_hex) => match asset {
                Asset::NativeAsset(_, name, _) => match self.convert_to_ascii {
                    true => String::from_utf8(name).ok(),
                    false => Some(hex::encode(name)),
                },
                _ => None,
            },
            _ => None,
        }
    }

    pub fn process_txo(
        &self,
        txo: &MultiEraOutput,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let asset_names: Vec<_> = txo
            .non_ada_assets()
            .into_iter()
            .filter_map(|x| self.to_string_output(x))
            .collect();

        if asset_names.is_empty() {
            return Ok(());
        }

        let address = txo.address().map(|x| x.to_string()).or_panic()?;

        for asset in asset_names {
            log::debug!("asset match found: ${asset}=>{address}");

            let crdt = model::CRDTCommand::any_write_wins(
                self.config.key_prefix.as_deref(),
                asset,
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
        rollback: bool,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if rollback {
            return Ok(());
        }

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
        let convert_to_ascii = self.convert_to_ascii.unwrap_or(false);
        let reducer = Reducer {
            config: self,
            convert_to_ascii,
        };

        super::Reducer::AddressByAsset(reducer)
    }
}
