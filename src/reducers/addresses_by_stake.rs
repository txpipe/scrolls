use pallas::ledger::addresses::{self, Address, StakeAddress};
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

fn any_address_to_stake_bech32(address: Address) -> Option<String> {
    match address {
        Address::Shelley(s) => match StakeAddress::try_from(s).ok() {
            Some(x) => x.to_bech32().ok(),
            _ => None,
        },
        Address::Byron(_) => None,
        Address::Stake(_) => None,
    }
}

impl Reducer {
    fn process_produced_txo(
        &mut self,
        tx: &MultiEraTx,
        tx_output: &MultiEraOutput,
        output_idx: usize,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        let tx_hash = tx.hash();
        let address = tx_output.address().or_panic()?;
        let stake_address = any_address_to_stake_bech32(address);

        let stake_address = match stake_address {
            Some(x) => x,
            None => return Ok(()),
        };

        if let Some(stake_addresses) = &self.config.filter {
            if let Err(_) = stake_addresses.binary_search(&stake_address) {
                return Ok(());
            }
        }

        let crdt = model::CRDTCommand::set_add(
            self.config.key_prefix.as_deref(),
            &stake_address,
            address.to_bech32()
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
            for (idx, produced) in tx.produces() {
                self.process_produced_txo(&tx, &produced, idx, output)?;
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

        super::Reducer::AddressesByStake(reducer)
    }
}

#[cfg(test)]
mod test {
    use super::any_address_to_stake_bech32;
    use pallas::ledger::addresses::Address;

    #[test]
    fn stake_bech32() {
        let addr = Address::from_bech32("addr1q86gknmykuldcngv0atyy56ex598p6m8f24nf9nmehmgpgfcmswqs6wnpls37lh7s3du977cxw67a9dpndnmafjs08asyqxe39").unwrap();
        let stake_bech32 = any_address_to_stake_bech32(addr).unwrap();
        assert_eq!(
            stake_bech32,
            "stake1uyudc8qgd8fslcgl0mlggk7zl0vr8d0wjksekea75eg8n7cw33m0s"
        );
    }
}
