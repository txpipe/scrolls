use crosscut::policies::*;
use gasket::error::AsWorkError;
use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::{MultiEraBlock, OutputRef};
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

        let key = match &self.config.key_prefix {
            Some(prefix) => prefix.to_string(),
            None => "balance_by_address".to_string(),
        };

        let crdt = model::CRDTCommand::PNCounter(key, output_tx.ada_amount() as i64);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    fn process_outbound_txo(
        &mut self,
        tx_output: &MultiEraOutput,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let address = tx_output.address(&self.address_hrp);

        if let Some(addresses) = &self.config.filter {
            if let Err(_) = addresses.binary_search(&address.to_string()) {
                return Ok(());
            }
        }

        let key = match &self.config.key_prefix {
            Some(prefix) => prefix.to_string(),
            None => "balance_by_address".to_string(),
        };

        let crdt = model::CRDTCommand::PNCounter(key, (-1) * tx_output.ada_amount() as i64);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
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

            for (_idx, tx_output) in tx.outputs().iter().enumerate() {
                self.process_outbound_txo(tx_output, output)?;
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

        super::Reducer::BalanceByAddress(reducer)
    }
}
