use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{crosscut, model, prelude::*};

use crate::crosscut::epochs::block_epoch;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
    pub aggr_by: Option<String>,
    pub filter: Option<crosscut::filters::Predicate>,
}

pub struct Reducer {
    config: Config,
    policy: crosscut::policies::RuntimePolicy,
    chain: crosscut::ChainWellKnownInfo,
}

impl Reducer {

    fn config_key(&self,
         epoch_no: u64,
         is_valid_trx: bool,
        ) -> String {
        let def_key_prefix = match is_valid_trx {
            true => "total_tx_count.valid",
            false => "total_tx_count.invalid"
        };
        
        match &self.config.aggr_by {
            Some(aggr) if aggr == "Epoch" => {
                let k = match &self.config.key_prefix {
                    Some(prefix) => format!("{}.{}", prefix, epoch_no),
                    None => format!("{}", def_key_prefix.to_string()),
                };

                return k;
            }
            _ => {
                let k = match &self.config.key_prefix {
                    Some(prefix) => format!("{}", prefix),
                    None => format!("{}", def_key_prefix.to_string()),
                };

                return k;
            }
        };
    }

    pub fn reduce_valid_tx(
        &mut self,
        epoch_no: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {

        let key = self.config_key(epoch_no, true);

        let crdt = model::CRDTCommand::PNCounter(key, 1);
        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_invalid_tx<'b>(
        &mut self,
        epoch_no: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let key = self.config_key(epoch_no, false);

        let crdt = model::CRDTCommand::PNCounter(key, 1);
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

                match tx.is_valid() {
                    true => self.reduce_valid_tx(epoch_no, output)?,
                    false => self.reduce_invalid_tx(epoch_no, output)?,
                };
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
            chain: chain.clone(),
            policy: policy.clone(),
        };

        super::Reducer::TotalTransactionsCount(reducer)
    }
}
