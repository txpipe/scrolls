use gasket::runtime::{spawn_stage, WorkOutcome};
use pallas::ledger::primitives::alonzo;
use pallas::ledger::primitives::alonzo::{PoolKeyhash, StakeCredential};
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
    input: InputPort,
    output: OutputPort,
    ops_count: gasket::metrics::Counter,
}

impl Worker {
    fn send_key_write(
        &mut self,
        cred: &StakeCredential,
        pool: &PoolKeyhash,
        slot: u64,
    ) -> Result<(), gasket::error::Error> {
        let hash = match cred {
            StakeCredential::AddrKeyhash(x) => x.to_string(),
            StakeCredential::Scripthash(x) => x.to_string(),
        };

        let key = match &self.config.key_prefix {
            Some(prefix) => format!("{}.{}", prefix, hash),
            None => hash.to_string(),
        };

        let value = pool.to_string();

        let crdt = model::CRDTCommand::LastWriteWins(key, value, slot);

        self.output.send(gasket::messaging::Message::from(crdt))?;
        self.ops_count.inc(1);

        Ok(())
    }

    fn reduce_alonzo_compatible_tx(
        &mut self,
        slot: u64,
        tx: &alonzo::TransactionBody,
    ) -> Result<(), gasket::error::Error> {
        tx.iter()
            .filter_map(|b| match b {
                alonzo::TransactionBodyComponent::Certificates(c) => Some(c),
                _ => None,
            })
            .flat_map(|c| c.iter())
            .filter_map(|c| match c {
                alonzo::Certificate::StakeDelegation(cred, pool) => Some((cred, pool)),
                _ => None,
            })
            .map(|(cred, pool)| self.send_key_write(cred, pool, slot))
            .collect()
    }

    fn reduce_block(&mut self, block: &model::MultiEraBlock) -> Result<(), gasket::error::Error> {
        match block {
            model::MultiEraBlock::Byron(_) => Ok(()),
            model::MultiEraBlock::AlonzoCompatible(block) => block
                .1
                .transaction_bodies
                .iter()
                .map(|tx| self.reduce_alonzo_compatible_tx(block.1.header.header_body.slot, tx))
                .collect(),
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
        pipeline.register_stage("pool_by_address", spawn_stage(self, Default::default()));
    }
}

impl Config {
    pub fn plugin(
        self,
        _chain: &crosscut::ChainWellKnownInfo,
        _intersect: &crosscut::IntersectConfig,
    ) -> super::Plugin {
        let worker = Worker {
            config: self,
            input: Default::default(),
            output: Default::default(),
            ops_count: Default::default(),
        };

        super::Plugin::PoolByStake(worker)
    }
}
