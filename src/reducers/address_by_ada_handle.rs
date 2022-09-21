use std::str::FromStr;

use pallas::codec::utils::{Bytes, KeyValuePairs};
use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::babbage::Value;
use pallas::ledger::traverse::MultiEraBlock;
use pallas::ledger::traverse::MultiEraOutput;
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
    policy_id: Hash<28>,
}

type Asset = (Hash<28>, Bytes, u64);

fn iter_policy_assets<'b>(
    policy: &'b Hash<28>,
    assets: &'b KeyValuePairs<Bytes, u64>,
) -> impl Iterator<Item = Asset> + 'b {
    assets
        .iter()
        .map(|(name, amount)| (policy.clone(), name.clone(), *amount))
}

// TODO: replace this with upstream pallas method
fn collect_txo_assets(txo: &MultiEraOutput) -> Vec<Asset> {
    txo.as_alonzo()
        .iter()
        .filter_map(|x| match &x.amount {
            Value::Multiasset(_, x) => Some(x),
            _ => None,
        })
        .flat_map(|x| x.iter())
        .flat_map(|(p, a)| iter_policy_assets(p, a))
        .collect::<Vec<_>>()
}

impl Reducer {
    fn to_ada_handle(&self, asset: Asset) -> Option<String> {
        let (policy, name, _) = asset;

        if !policy.eq(&self.policy_id) {
            return None;
        }

        String::from_utf8(name.into()).ok()
    }

    pub fn process_txo(
        &self,
        txo: &MultiEraOutput,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let handles: Vec<_> = collect_txo_assets(&txo)
            .into_iter()
            .filter_map(|x| self.to_ada_handle(x))
            .collect();

        if handles.is_empty() {
            return Ok(());
        }

        let address = txo.address().map(|x| x.to_string()).or_panic()?;

        for handle in handles {
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
        let reducer = Reducer {
            policy_id: Hash::<28>::from_str(&self.policy_id_hex)
                .expect("invalid ada handle policy id"),
            config: self,
        };

        super::Reducer::AddressByAdaHandle(reducer)
    }
}
