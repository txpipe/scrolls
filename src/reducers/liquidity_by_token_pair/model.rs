use lazy_static::__Deref;
use pallas::ledger::primitives::babbage::{AssetName, PlutusData, PolicyId};
use std::fmt;

use super::utils::policy_id_from_str;

#[derive(PartialEq, Debug)]
pub struct TokenPair {
    pub coin_a: PoolAsset,
    pub coin_b: PoolAsset,
}

impl TryFrom<&PlutusData> for TokenPair {
    type Error = ();

    fn try_from(value: &PlutusData) -> Result<Self, Self::Error> {
        match value {
            PlutusData::Constr(pd) => {
                let _pd1 = pd.fields.get(0).ok_or(())?;
                let _pd2 = pd.fields.get(1).ok_or(())?;

                return match (
                    PoolAsset::try_from(_pd1).ok(),
                    PoolAsset::try_from(_pd2).ok(),
                ) {
                    (Some(coin_a), Some(coin_b)) => Ok(Self { coin_a, coin_b }),
                    _ => Err(()),
                };
            }
            _ => Err(()),
        }
    }
}

#[derive(core::cmp::PartialOrd)]
pub enum PoolAsset {
    Ada,
    AssetClass(PolicyId, AssetName),
}

impl TryFrom<&PlutusData> for PoolAsset {
    type Error = ();

    fn try_from(value: &PlutusData) -> Result<Self, Self::Error> {
        if let PlutusData::Constr(pd) = value {
            return match (pd.fields.get(0), pd.fields.get(1)) {
                (
                    Some(PlutusData::BoundedBytes(currency_symbol)),
                    Some(PlutusData::BoundedBytes(token_name)),
                ) => {
                    if currency_symbol.len() == 0 && token_name.len() == 0 {
                        return Ok(PoolAsset::Ada);
                    } else {
                        let cs_clone = &mut (&mut currency_symbol.deref().clone());
                        let pid = policy_id_from_str(&cs_clone);
                        return Ok(PoolAsset::AssetClass(
                            pid,
                            AssetName::from(token_name.to_vec()),
                        ));
                    }
                }
                _ => Err(()),
            };
        }

        Err(())
    }
}

impl std::fmt::Debug for PoolAsset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.clone() {
            PoolAsset::Ada => write!(f, "Ada"),
            &PoolAsset::AssetClass(pid, tkn) => {
                write!(
                    f,
                    "AssetClass {{ policy: '{}', name: '{}' }}",
                    hex::encode(pid.to_ascii_lowercase()),
                    hex::encode(tkn.to_vec())
                )
            }
        }
    }
}

impl PartialEq for PoolAsset {
    fn eq(&self, other: &Self) -> bool {
        match (&self.clone(), &other.clone()) {
            (&PoolAsset::Ada, &PoolAsset::Ada) => true,
            (&PoolAsset::AssetClass(a_pid, a_tkn), &PoolAsset::AssetClass(b_pid, b_tkn)) => {
                a_pid.deref() == b_pid.deref() && a_tkn == b_tkn
            }
            _ => false,
        }
    }
}
#[derive(core::cmp::PartialOrd, Debug)]
pub struct AssetClass {
    currency_symbol: PolicyId,
    asset_name: AssetName,
}

impl std::fmt::Display for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AssetClass {{ policy: '{}', name: '{}' }}",
            hex::encode(self.currency_symbol.to_vec()),
            hex::encode(self.asset_name.to_vec())
        )
    }
}

impl PartialEq for AssetClass {
    fn eq(&self, other: &Self) -> bool {
        self.currency_symbol == other.currency_symbol && self.asset_name == other.asset_name
    }
}