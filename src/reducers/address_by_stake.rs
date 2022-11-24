use pallas::ledger::addresses::Address::Shelley;
use pallas::ledger::addresses::{Error, StakeAddress};
use pallas::ledger::traverse::MultiEraBlock;
use serde::Deserialize;

use crate::{model, prelude::*};

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
}

impl Reducer {
    pub fn reduce_block<'b>(
        &mut self,
        block: &'b MultiEraBlock<'b>,
        ctx: &model::BlockContext,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        for tx in block.txs().into_iter() {
            for (_, out) in tx.produces().into_iter() {
                let address = out.address().or_panic()?;
                match address {
                    Shelley(shelly_address) => {
                        let stake_address_result: Result<StakeAddress, Error> =
                            shelly_address.clone().try_into();
                        if stake_address_result.is_ok() {
                            let stake_address =
                                stake_address_result.unwrap().to_bech32().or_panic()?;
                            let payment_address = shelly_address.to_bech32().or_panic()?;

                            let crdt = model::CRDTCommand::set_add(
                                self.config.key_prefix.as_deref(),
                                &stake_address,
                                payment_address,
                            );
                            output.send(crdt.into());
                        }
                    }
                    _ => (),
                };
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let reducer = Reducer { config: self };

        super::Reducer::AddressByStake(reducer)
    }
}
