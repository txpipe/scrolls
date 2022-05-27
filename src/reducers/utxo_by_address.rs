use gasket::{
    error::AsWorkError,
    runtime::{spawn_stage, WorkOutcome},
};
use pallas::crypto::hash::Hash;
use pallas::ledger::primitives::{alonzo, byron};
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

type InputPort = gasket::messaging::InputPort<model::ChainSyncCommandEx>;
type OutputPort = gasket::messaging::OutputPort<model::CRDTCommand>;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Worker {
    config: Config,
    address_hrp: String,
    input: InputPort,
    output: OutputPort,
    ops_count: gasket::metrics::Counter,
}

impl Worker {
    fn send_set_add(
        &mut self,
        address: &str,
        tx_hash: Hash<32>,
        tx_idx: usize,
    ) -> Result<(), gasket::error::Error> {
        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, address),
            None => address.to_string(),
        };

        let member = format!("{}:{}", tx_hash, tx_idx);
        let crdt = model::CRDTCommand::TwoPhaseSetAdd(key, member);

        self.output.send(gasket::messaging::Message::from(crdt))?;
        self.ops_count.inc(1);

        Ok(())
    }

    fn reduce_byron_tx(&mut self, tx: &byron::TxPayload) -> Result<(), gasket::error::Error> {
        let tx_hash = tx.transaction.to_hash();

        tx.transaction
            .outputs
            .iter()
            .enumerate()
            .map(move |(tx_idx, tx)| {
                let address = tx.address.to_addr_string().or_work_err()?;
                self.send_set_add(&address, tx_hash, tx_idx)
            })
            .collect()
    }

    fn reduce_alonzo_compatible_tx(
        &mut self,
        tx: &alonzo::TransactionBody,
    ) -> Result<(), gasket::error::Error> {
        let tx_hash = tx.to_hash();

        tx.iter()
            .filter_map(|b| match b {
                alonzo::TransactionBodyComponent::Outputs(o) => Some(o),
                _ => None,
            })
            .flat_map(|o| o.iter())
            .enumerate()
            .map(move |(tx_idx, output)| {
                let address = output.to_bech32_address(&self.address_hrp).or_work_err()?;
                self.send_set_add(&address, tx_hash, tx_idx)
            })
            .collect()
    }

    fn reduce_block(&mut self, block: &model::MultiEraBlock) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(byron::Block::MainBlock(x)) => x
                .body
                .tx_payload
                .iter()
                .map(|tx| self.reduce_byron_tx(tx))
                .collect(),
            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(x) => {
                x.1.transaction_bodies
                    .iter()
                    .map(|tx| self.reduce_alonzo_compatible_tx(tx))
                    .collect()
            }
        }
    }
}

impl gasket::runtime::Worker for Worker {
    fn metrics(&self) -> gasket::metrics::Registry {
        gasket::metrics::Builder::new()
            .with_counter("ops_count", &self.ops_count)
            .build()
    }

    fn work(&mut self) -> gasket::runtime::WorkResult {
        let msg = self.input.recv()?;

        match msg.payload {
            model::ChainSyncCommandEx::RollForward(block) => self.reduce_block(&block)?,
            model::ChainSyncCommandEx::RollBack(point) => {
                log::warn!("rollback requested for {:?}", point);
            }
        }

        Ok(WorkOutcome::Partial)
    }
}

impl super::Pluggable for Worker {
    fn borrow_input_port(&mut self) -> &'_ mut InputPort {
        &mut self.input
    }

    fn borrow_output_port(&mut self) -> &'_ mut OutputPort {
        &mut self.output
    }

    fn spawn(self, pipeline: &mut bootstrap::Pipeline) {
        pipeline.register_stage("utxo_by_address", spawn_stage(self, Default::default()));
    }
}

impl Config {
    pub fn plugin(
        self,
        chain: &crosscut::ChainWellKnownInfo,
        _intersect: &crosscut::IntersectConfig,
    ) -> super::Plugin {
        let worker = Worker {
            config: self,
            address_hrp: chain.address_hrp.clone(),
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
        };

        super::Plugin::UtxoByAddress(worker)
    }
}
