use lazy_static::__Deref;
use pallas::ledger::primitives::babbage::{AssetName, PlutusData, PolicyId};
use std::fmt;

use super::utils::policy_id_from_str;

#[derive(core::cmp::PartialOrd)]
pub enum PoolAsset {
    Ada,
    AssetClass(PolicyId, AssetName),
}

impl PoolAsset {
    pub fn from_plutus_data(plutus_data: &PlutusData) -> Option<Self> {
        match plutus_data {
            PlutusData::Constr(pd) => match (pd.fields.get(0), pd.fields.get(1)) {
                (Some(PlutusData::BoundedBytes(cs)), Some(PlutusData::BoundedBytes(tkn))) => {
                    if cs.len() == 0 && tkn.len() == 0 {
                        Some(PoolAsset::Ada)
                    } else {
                        let cs_clone = &mut (cs.deref().clone());
                        let pid = policy_id_from_str(&cs_clone);
                        Some(PoolAsset::AssetClass(pid, AssetName::from(tkn.to_vec())))
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }
}

impl std::fmt::Debug for PoolAsset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.clone() {
            PoolAsset::Ada => write!(f, "Ada"),
            &PoolAsset::AssetClass(pid, tkn) => {
                write!(
                    f,
                    "AssetClass(policy: {}, name: {})",
                    hex::encode(pid.to_ascii_lowercase()),
                    hex::encode(tkn.to_vec())
                )
            }
        }
    }
}

impl std::fmt::Display for PoolAsset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PoolAsset::Ada => write!(f, "Ada"),
            _ => write!(f, "{:?}", self),
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
#[derive(core::cmp::PartialOrd)]
pub struct AssetClass {
    currency_symbol: PolicyId,
    asset_name: AssetName,
}

impl std::fmt::Debug for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AssetClass (policy: {}, name: {})",
            hex::encode(self.currency_symbol.to_vec()),
            hex::encode(self.asset_name.to_vec())
        )
    }
}

impl std::fmt::Display for AssetClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AssetClass (policy: {}, name: {})",
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
