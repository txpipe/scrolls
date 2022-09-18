use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{MultiEraBlock, OutputRef, Subject};
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

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
    fn process_consumed_txo(
        &mut self,
        ctx: &model::BlockContext,
        input: &OutputRef,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let utxo = ctx.find_utxo(input).apply_policy(&self.policy).or_panic()?;

        let utxo = match utxo {
            Some(x) => x,
            None => return Ok(()),
        };

        let address = utxo.address().map(|addr| addr.to_string()).or_panic()?;

        for asset in utxo.assets().iter() {
            let sub = &asset.subject;
            let quantity = &asset.quantity;

            let delta = *quantity as i64 * (-1);

            let prefix = match &self.config.key_prefix {
                Some(prefix) => prefix,
                None => "asset_holders_by_asset",
            };

            match sub {
                Subject::NativeAsset(policy_id, asset_name) =>  {
                    let asset_id = format!("{}{}", policy_id, asset_name);

                    let key = format!("{}.{}", prefix.to_string(), asset_id);

                    let crdt = model::CRDTCommand::SortedSetRemove(key, address.to_string(), delta);

                    output.send(gasket::messaging::Message::from(crdt))?;
                }
                _ => {},
            };
        }

        Ok(())
    }

    fn process_produced_txo(
        &mut self,
        tx_output: &MultiEraOutput,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let address = tx_output.address().map(|addr| addr.to_string()).or_panic()?;

        for asset in tx_output.assets().iter() {
            let sub = &asset.subject;
            let quantity = &asset.quantity;

            let prefix = match &self.config.key_prefix {
                Some(prefix) => prefix,
                None => "asset_holders_by_asset",
            };

            let delta = *quantity as i64;

            match sub {
                Subject::NativeAsset(policy_id, asset_name) =>  {
                    let asset_id = format!("{}{}", policy_id, asset_name);

                    let key = format!("{}.{}", prefix, asset_id);

                    let crdt = model::CRDTCommand::SortedSetAdd(key, address.to_string(), delta);

                    output.send(gasket::messaging::Message::from(crdt))?;
                }
                _ => {},
            };
        }

        Ok(())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            if filter_matches!(self, block, &tx, ctx) {

                for consumed in tx.consumes().iter().map(|i| i.output_ref()) {
                    self.process_consumed_txo(&ctx, &consumed, output)?;
                }

                for produced in tx.produces().iter() {
                    self.process_produced_txo(produced, output)?;
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

        super::Reducer::AssetHoldersByAsset(reducer)
    }
}

// How to query
// 127.0.0.1:6379> ZRANGEBYSCORE "c14.5d9d887de76a2c9d057b3e5d34d5411f7f8dc4d54f0c06e8ed2eb4a9494e4459" 1 +inf
// 1) "addr1q8lmu79hgm3sppz8dta3aftf0cwh2v2eja56wqvzqy4jj0zjt7qgvj7saxdxve35c4ehuxuam4czlz9fw6ls7zr4as9s609d7u"

// TODO empty (0 scores need to be regularly garbage collected either by scrolls or by client app [hacky])

// Wing Riders - L token holders
// ZRANGEBYSCORE "c14.5d9d887de76a2c9d057b3e5d34d5411f7f8dc4d54f0c06e8ed2eb4a9494e44594C" 1 +inf

// gc command
// ZREMRANGEBYSCORE "c14.fff75ea36607458cc1f924fb1a9d4dbf53afb475357cc81c04b420be5175697a486f6c6f436172645331" 0 0

// 127.0.0.1:6379> get _cursor
// "49907420,a4b89845e2224de34701806a3d5768f3c0efd0c4ce44bc0f19127d3588659eb4"