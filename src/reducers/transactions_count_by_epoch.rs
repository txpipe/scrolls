use serde::Deserialize;

use crate::{
    crosscut::{self, EpochCalculator},
    model,
};
use pallas::ledger::primitives::byron;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
    shelley_known_slot: u64,
    shelley_epoch_length: u64,
    byron_epoch_length: u64,
    byron_slot_length: u64,
}

impl Reducer {
    fn reduce_alonzo_compatible_tx(
        &mut self,
        slot: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let epoch_no = EpochCalculator::get_shelley_epoch_no_for_absolute_slot(
            self.shelley_known_slot,
            self.shelley_epoch_length,
            slot,
        );

        return self.increment_key(epoch_no, output);
    }

    fn reduce_byron_compatible_tx(
        &mut self,
        slot: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let epoch_no = EpochCalculator::get_byron_epoch_no_for_absolute_slot(
            self.byron_epoch_length,
            self.byron_slot_length,
            slot,
        );

        return self.increment_key(epoch_no, output);
    }

    fn increment_key(
        &mut self,
        epoch_no: u64,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let prefix = match &self.config.key_prefix {
            Some(prefix) => prefix,
            None => "transactions_by_epoch",
        };

        let key = format!("{}.{}", prefix, epoch_no.to_string());

        let crdt = model::CRDTCommand::PNCounter(key, 1);

        output.send(gasket::messaging::Message::from(crdt))?;

        Ok(())
    }

    pub fn reduce_block(
        &mut self,
        block: &model::MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(byron::Block::MainBlock(x)) => x
                .body
                .tx_payload
                .iter()
                .map(|_tx| {
                    self.reduce_byron_compatible_tx(x.header.consensus_data.0.to_abs_slot(), output)
                })
                .collect(),

            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(x) => x
                .1
                .transaction_bodies
                .iter()
                .map(|_tx| self.reduce_alonzo_compatible_tx(x.1.header.header_body.slot, output))
                .collect(),
        }
    }
}

impl Config {
    pub fn plugin(self, chain: &crosscut::ChainWellKnownInfo) -> super::Reducer {
        let reducer = Reducer {
            config: self,
            shelley_known_slot: chain.shelley_known_slot.clone() as u64,
            shelley_epoch_length: chain.shelley_epoch_length.clone() as u64,
            byron_epoch_length: chain.byron_epoch_length.clone() as u64,
            byron_slot_length: chain.byron_slot_length.clone() as u64,
        };

        super::Reducer::TransactionsCountByEpoch(reducer)
    }
}
