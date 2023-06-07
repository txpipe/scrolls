
use pallas::ledger::traverse::MultiEraOutput;
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

use crate::crosscut::epochs::block_epoch;

#[derive(Deserialize, Copy, Clone)]
pub enum Projection {
    Individual,
    Total,
}

#[derive(Deserialize, Copy, Clone, PartialEq)]
pub enum AggrType {
    Epoch,
}

#[derive(Deserialize, Copy, Clone, PartialEq)]
pub enum AddrType {
    Hex,
}

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub filter: Option<crosscut::filters::Predicate>,
    pub aggr_by: Option<AggrType>,
    pub key_addr_type: Option<AddrType>,
    pub projection: Projection,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    chain: crosscut::ChainWellKnownInfo,
}

impl Reducer {

    fn config_key(&self, address: String, epoch_no: u64) -> String {
        let def_key_prefix = "volume_by_address";

        match &self.config.aggr_by {
            Some(aggr_type) if matches!(aggr_type, AggrType::Epoch) => {
                return match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}.{}", prefix, address, epoch_no),
                    None => format!("{}.{}", def_key_prefix.to_string(), address),
                };
            },
            _ => {
                return match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}", prefix, address),
                    None => format!("{}.{}", def_key_prefix.to_string(), address),
                };
            }
        };
    }

    fn process_produced_txo(
        &mut self,
        tx_output: &MultiEraOutput,
        output: &mut super::OutputPort,
        epoch_no: u64
    ) -> Result<(), gasket::error::Error> {

        let address = tx_output.address()
        .map(|addr| {
            match &self.config.key_addr_type {
                Some(addr_typ) if matches!(addr_typ, AddrType::Hex) => addr.to_hex(),
                _ => addr.to_string()
            }
        })
        .or_panic()?;

        let key = self.config_key(address, epoch_no);

        let amount = tx_output.lovelace_amount() as i64;

        let crdt = match &self.config.projection {
            Projection::Individual => model::CRDTCommand::GrowOnlySetAdd(key, format!("{}", amount)),
            Projection::Total => model::CRDTCommand::PNCounter(key, amount as i64),
        };

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
            if filter_matches!(self, block, &tx, ctx) {
                let epoch_no = block_epoch(&self.chain, block);

                for (_, produced) in tx.produces() {
                    self.process_produced_txo(&produced, output, epoch_no)?;
                }

            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self,
        chain: &crosscut::ChainWellKnownInfo,
        policy: &crosscut::policies::RuntimePolicy) -> super::Reducer {

        let reducer = Reducer {
            config: self,
            policy: policy.clone(),
            chain: chain.clone(),
        };

        super::Reducer::VolumeByAddress(reducer)
    }
}
