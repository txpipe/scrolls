use pallas::ledger::traverse::{MultiEraBlock, OutputRef};
use pallas::ledger::traverse::{MultiEraOutput, MultiEraTx};
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

        let address = utxo.address().map(|x| x.to_string()).or_panic()?;

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, address),
            None => format!("{}.{}", "balance_by_address".to_string(), address),
        };

        let crdt = model::CRDTCommand::PNCounter(key, -1 * utxo.ada_amount() as i64);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    fn process_outbound_txo(
        &mut self,
        tx_output: &MultiEraOutput,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let address = tx_output.address().map(|x| x.to_string()).or_panic()?;

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, address),
            None => format!("{}.{}", "balance_by_address".to_string(), address),
        };

        let crdt = model::CRDTCommand::PNCounter(key, tx_output.ada_amount() as i64);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_valid_tx(
        &mut self,
        tx: &MultiEraTx,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for input in tx.inputs().iter().map(|i| i.output_ref()) {
            self.process_inbound_txo(&ctx, &input, output)?;
        }

        for (_idx, tx_output) in tx.outputs().iter().enumerate() {
            self.process_outbound_txo(tx_output, output)?;
        }

        Ok(())
    }

    pub fn reduce_invalid_tx<'b>(
        &mut self,
        tx: &MultiEraTx,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for input in tx.collateral().iter().map(|i| i.output_ref()) {
            self.process_inbound_txo(&ctx, &input, output)?;
        }

        for (_idx, tx_output) in tx.outputs().iter().enumerate() {
            self.process_outbound_txo(tx_output, output)?;
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
                match tx.is_valid() {
                    true => self.reduce_valid_tx(&tx, ctx, output)?,
                    false => self.reduce_invalid_tx(&tx, ctx, output)?,
                };
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

        super::Reducer::BalanceByAddress(reducer)
    }
}
