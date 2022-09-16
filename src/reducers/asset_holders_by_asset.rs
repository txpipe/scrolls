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

        let address = utxo.address().map(|x| x.to_string()).or_panic()?;

        for asset in utxo.assets().iter() {
            let sub = &asset.subject;
            let quantity = &asset.quantity;

            let delta = *quantity as i64 * -1;

            match sub {
                Subject::NativeAsset(policy_id, asset_name) =>  {
                    let asset_name_hex = format!("{}{:x?}", policy_id, asset_name);

                    let crdt = model::CRDTCommand::SortedSetRemove(asset_name_hex, address.to_string(), delta);

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
        let address = tx_output.address().map(|x| x.to_string()).or_panic()?;

        for asset in tx_output.assets().iter() {
            let sub = &asset.subject;
            let quantity = &asset.quantity;

            match sub {
                Subject::NativeAsset(policy_id, asset_name) =>  {
                    let asset_name_hex = format!("{}{:x?}", policy_id, asset_name);

                    let delta = *quantity as i64;

                    let crdt = model::CRDTCommand::SortedSetAdd(asset_name_hex, address.to_string(), delta);

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
