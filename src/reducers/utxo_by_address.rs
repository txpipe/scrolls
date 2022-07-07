use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{MultiEraBlock, MultiEraTx, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
}

impl Reducer {
    fn process_inbound_txo(
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

        let address = utxo.to_address().and_then(|x| x.to_bech32()).or_panic()?;

        if let Some(addresses) = &self.config.filter {
            if let Err(_) = addresses.binary_search(&address) {
                return Ok(());
            }
        }

        let crdt = model::CRDTCommand::set_remove(
            self.config.key_prefix.as_deref(),
            &address,
            input.to_string(),
        );

        output.send(crdt.into())
    }

    fn process_outbound_txo(
        &mut self,
        tx: &MultiEraTx,
        tx_output: &MultiEraOutput,
        output_idx: usize,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let tx_hash = tx.hash();
        let address = tx_output
            .to_address()
            .and_then(|x| x.to_bech32())
            .or_panic()?;

        if let Some(addresses) = &self.config.filter {
            if let Err(_) = addresses.binary_search(&address) {
                return Ok(());
            }
        }

        let crdt = model::CRDTCommand::set_add(
            self.config.key_prefix.as_deref(),
            &address,
            format!("{}#{}", tx_hash, output_idx),
        );

        output.send(crdt.into())
    }

    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            for input in tx.inputs().iter().filter_map(|i| i.output_ref()) {
                self.process_inbound_txo(&ctx, &input, output)?;
            }

            for (idx, tx_output) in tx.outputs().iter().enumerate() {
                self.process_outbound_txo(&tx, tx_output, idx, output)?;
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

        super::Reducer::UtxoByAddress(reducer)
    }
}
