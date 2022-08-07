use serde::Deserialize;

use pallas::ledger::traverse::{Feature, MultiEraBlock};

use crate::model;

#[derive(Deserialize)]
pub struct Config {
    pub key_prefix: Option<String>,
}

pub struct Reducer {
    config: Config,
}

impl Reducer {
    pub fn reduce_block(
        &mut self,
        block: &MultiEraBlock,
        output: &mut super::OutputPort,
    ) -> Result<(), gasket::error::Error> {
        if block.era().has_feature(Feature::MultiAssets) {
            for tx in block.txs() {
                let mint = tx.mint();

                if let Some(mints) = mint.as_alonzo() {
                    for (policy, assets) in mints.iter() {
                        let policy_id = hex::encode(policy.as_slice());

                        let number_of_minted_or_destroyed = assets.len();

                        let key = match &self.config.key_prefix {
                            Some(prefix) => format!("{}.{}", prefix, policy_id),
                            None => format!(
                                "{}.{}",
                                "transaction_count_by_native_token_policy".to_string(),
                                policy_id
                            ),
                        };

                        let crdt = model::CRDTCommand::PNCounter(
                            key,
                            number_of_minted_or_destroyed as i64,
                        );
                        output.send(gasket::messaging::Message::from(crdt))?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Config {
    pub fn plugin(self) -> super::Reducer {
        let reducer = Reducer { config: self };

        super::Reducer::TxCountByNativeTokenPolicyId(reducer)
    }
}
