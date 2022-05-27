use gasket::{
    runtime::{spawn_stage, WorkOutcome},
};
use pallas::{ledger::primitives::alonzo};
use serde::Deserialize;

use crate::{bootstrap, crosscut, model};

use std::collections::HashSet;

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


    fn increment_for_addresses(
        &mut self,
        contract_addresses: &std::collections::HashSet<String>,
    ) -> Result<(), gasket::error::Error> {

        for contract_address in contract_addresses {
            let key = match &self.config.key_prefix {
                Some(prefix) => format!("{}.{}", prefix, contract_address.to_string()),
                None => format!("{}", contract_address.to_string()),
            };
    
            let crdt = model::CRDTCommand::PNCounter(key, "1".to_string());
            self.output.send(gasket::messaging::Message::from(crdt))?;
            self.ops_count.inc(1);
        }

        Ok(())
    }

    fn reduce_alonzo_compatible_tx(
        &mut self,
        tx: &alonzo::TransactionBody,
    ) -> Result<(), gasket::error::Error> {
        let hrp_addr = &self.address_hrp.clone();

        let addresses: Vec<Option<String>> = tx.iter()
            .filter_map(|b| match b {
                alonzo::TransactionBodyComponent::Outputs(o) => Some(o),
                _ => None,
            })
            .flat_map(|o| o.iter())
            .enumerate()
            .map(move |(_tx_idx, output)| {
                let address = output.to_bech32_address(hrp_addr).unwrap();

                fn get_bit_at(input: u8, n: u8) -> bool {
                    if n < 32 {
                        input & (1 << n) != 0
                    } else {
                        false
                    }
                }
            
                // first byte of address is header
                let first_byte_of_address = output.address.as_slice()[0];
                // https://github.com/input-output-hk/cardano-ledger/blob/master/eras/alonzo/test-suite/cddl-files/alonzo.cddl#L135
                let is_smart_contract_address = get_bit_at(first_byte_of_address, 4);

                if is_smart_contract_address {
                    return Some(address);
                }

                return None::<String>;
            }).collect();

            if addresses.len() == 0 {
                return Result::Ok(());
            }

            let currated_addresses: Vec<String> = addresses
                .into_iter()
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect();

            let deduped_addresses: HashSet<String> = HashSet::from_iter(currated_addresses);

            return self.increment_for_addresses(&deduped_addresses);
        }

    fn reduce_block(&mut self, block: &model::MultiEraBlock) -> Result<(), gasket::error::Error> {
        match block {
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
        pipeline.register_stage("transactions_count_by_contract_address", spawn_stage(self, Default::default()));
    }
}

impl super::IntoPlugin for Config {
    fn plugin(
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

        super::Plugin::TransactionsCountByContractAddress(worker)
    }
}
