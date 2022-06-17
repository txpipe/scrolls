use crosscut::policies::*;
use gasket::error::AsWorkError;
use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{MultiEraBlock, MultiEraTx, OutputRef};
use serde::Deserialize;

use crate::{crosscut, model};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<Vec<String>>,
    pub policy: Option<ReducerPolicy>,
}

pub struct Reducer {
    config: Config,
    address_hrp: String,
}

impl Reducer {
    fn process_inbound_txo(
        &mut self,
        ctx: &model::BlockContext,
        input: &OutputRef,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let output_idx = input.tx_index();

        let inbound_tx = ctx
            .find_ref_tx(input.tx_id())
            .apply_policy(&self.config.policy)
            .or_work_err()?;

        let inbound_tx = match inbound_tx {
            Some(x) => x,
            None => return Ok(()),
        };

        let output_tx = inbound_tx
            .output_at(input.tx_index() as usize)
            .ok_or(crate::Error::ledger("output index not found in tx"))
            .apply_policy(&self.config.policy)
            .or_work_err()?;

        let output_tx = match output_tx {
            Some(x) => x,
            None => return Ok(()),
        };

        let address = output_tx.address(&self.address_hrp);

        if let Some(addresses) = &self.config.filter {
            if let Err(_) = addresses.binary_search(&address.to_string()) {
                return Ok(());
            }
        }

        let crdt = model::CRDTCommand::set_remove(
            self.config.key_prefix.as_deref(),
            &address,
            format!("{}#{}", inbound_tx.hash(), output_idx),
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
        let address = tx_output.address(&self.address_hrp);

        if let Some(addresses) = &self.config.filter {
            if let Err(_) = addresses.binary_search(&address.to_string()) {
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
                self.process_inbound_txo(&ctx, &input, output)
                    .or_work_err()?;
            }

            for (idx, tx_output) in tx.outputs().iter().enumerate() {
                self.process_outbound_txo(&tx, tx_output, idx, output)?;
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self, chain: &crosscut::ChainWellKnownInfo) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            address_hrp: chain.address_hrp.clone(),
        };

        super::Reducer::UtxoByAddress(reducer)
    }
}
