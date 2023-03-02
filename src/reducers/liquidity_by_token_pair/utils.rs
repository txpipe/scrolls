use pallas::ledger::{
    primitives::babbage::{AssetName, PolicyId},
    traverse::Asset,
};

use super::model::PoolAsset;

pub fn policy_id_from_str(str: &Vec<u8>) -> PolicyId {
    let mut pid: [u8; 28] = [0; 28];
    for (i, b) in str.iter().enumerate() {
        if i >= 28 {
            break;
        }
        pid[i] = *b;
    }

    PolicyId::from(pid)
}

pub fn contains_currency_symbol(currency_symbol: &String, assets: &Vec<Asset>) -> bool {
    assets.iter().any(|asset| {
        asset
            .policy_hex()
            .or(Some(String::new())) // in case ADA is part of the vector
            .unwrap()
            .as_str()
            .eq(currency_symbol)
    })
}

pub fn filter_by_native_fungible(assets: Vec<Asset>) -> Vec<Asset> {
    let mut result: Vec<Asset> = Vec::new();
    for asset in assets {
        match asset {
            Asset::NativeAsset(_, _, quantity) if quantity > 1 => {
                result.push(asset);
            }
            _ => (),
        }
    }
    result
}

pub fn pool_asset_from(hex_currency_symbol: &String, hex_asset_name: &String) -> Option<PoolAsset> {
    if hex_currency_symbol.len() == 0 && hex_asset_name.len() == 0 {
        return Some(PoolAsset::Ada);
    }

    if let (Some(pid), Some(tkn)) = (
        hex::decode(hex_currency_symbol).ok(),
        hex::decode(hex_asset_name).ok(),
    ) {
        return Some(PoolAsset::AssetClass(
            policy_id_from_str(&pid),
            AssetName::from(tkn),
        ));
    }

    None
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use pallas::ledger::{primitives::babbage::PolicyId, traverse::Asset};

    use crate::reducers::liquidity_by_token_pair::utils::{
        contains_currency_symbol, filter_by_native_fungible,
    };

    static CURRENCY_SYMBOL_1: &str = "93744265ed9762d8fa52c4aacacc703aa8c81de9f6d1a59f2299235b";
    static CURRENCY_SYMBOL_2: &str = "158fd94afa7ee07055ccdee0ba68637fe0e700d0e58e8d12eca5be46";

    fn mock_assets() -> Vec<Asset> {
        [
            Asset::NativeAsset(
                PolicyId::from_str(CURRENCY_SYMBOL_1).ok().unwrap(),
                "Tkn1".to_string().as_bytes().to_vec(),
                1,
            ),
            Asset::NativeAsset(
                PolicyId::from_str(CURRENCY_SYMBOL_1).ok().unwrap(),
                "Tkn2".to_string().as_bytes().to_vec(),
                2,
            ),
            Asset::NativeAsset(
                PolicyId::from_str(CURRENCY_SYMBOL_2).ok().unwrap(),
                "Tkn3".to_string().as_bytes().to_vec(),
                3,
            ),
        ]
        .to_vec()
    }

    #[test]
    fn test_contains_currency_symbol() {
        let mock_assets = mock_assets();
        assert_eq!(
            contains_currency_symbol(&CURRENCY_SYMBOL_1.to_string(), &mock_assets),
            true
        );
        assert_eq!(
            contains_currency_symbol(&"".to_string(), &mock_assets),
            false
        );
        assert_eq!(
            contains_currency_symbol(&"123abc".to_string(), &mock_assets),
            false
        );
    }

    #[test]
    fn test_filter_by_native_fungible() {
        let filtered = filter_by_native_fungible(mock_assets());
        assert_eq!(filtered.len(), 2);
        assert_eq!(
            filtered.get(0).unwrap().ascii_name(),
            Some(String::from("Tkn2"))
        );
    }
}
