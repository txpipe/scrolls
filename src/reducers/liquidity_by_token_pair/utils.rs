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

    use pallas::ledger::{
        primitives::{
            babbage::{PlutusData, PolicyId},
            Fragment,
        },
        traverse::Asset,
    };

    use crate::reducers::liquidity_by_token_pair::{
        model::{PoolAsset, TokenPair},
        utils::contains_currency_symbol,
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
    fn test_valid_key_for_ada_min() {
        let hex_pool_datum = "d8799fd8799f4040ffd8799f581c29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6434d494eff1b00004ce6fb73282200d87a80ff";
        let data = hex::decode(hex_pool_datum).unwrap();
        let plutus_data = PlutusData::decode_fragment(&data).unwrap();
        let token_pair = TokenPair::try_from(&plutus_data).unwrap();
        let key = token_pair.key();
        assert_eq!(true, key.is_some());
        assert_eq!(
            "29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6.4d494e",
            key.unwrap()
        )
    }

    #[test]
    fn test_valid_key_for_min_ada() {
        let hex_pool_datum = "d8799fd8799f4040ffd8799f581c29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6434d494eff1b00004ce6fb73282200d87a80ff";
        let data = hex::decode(hex_pool_datum).unwrap();
        let plutus_data = PlutusData::decode_fragment(&data).unwrap();
        let tp = TokenPair::try_from(&plutus_data).unwrap();
        let token_pair = TokenPair {
            coin_a: tp.coin_b,
            coin_b: tp.coin_a,
        };
        let key = token_pair.key();
        assert_eq!(true, key.is_some());
        assert_eq!(
            "29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6.4d494e",
            key.unwrap()
        )
    }

    #[test]
    fn test_valid_key_for_same_assets() {
        let hex_pool_datum = "d8799fd8799f4040ffd8799f581c29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6434d494eff1b00004ce6fb73282200d87a80ff";
        let data = hex::decode(hex_pool_datum).unwrap();
        let plutus_data = PlutusData::decode_fragment(&data).unwrap();
        let tp1 = TokenPair::try_from(&plutus_data).unwrap();
        let tp2 = TokenPair::try_from(&plutus_data).unwrap();

        let tp1_invalid = TokenPair {
            coin_a: tp1.coin_b,
            coin_b: tp2.coin_b,
        };
        let key1 = tp1_invalid.key();
        assert_eq!(true, key1.is_none());

        let tp2_invalid = TokenPair {
            coin_a: tp1.coin_a,
            coin_b: tp2.coin_a,
        };
        let key2 = tp2_invalid.key();
        assert_eq!(true, key2.is_none());
    }

    #[test]
    fn test_valid_key_for_min_djed() {
        let hex_pool_datum = "d8799fd8799f581c29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6434d494effd8799f581c8db269c3ec630e06ae29f74bc39edd1f87c819f1056206e879a1cd614c446a65644d6963726f555344ff1b000000012d9b96321b000000012dc40542d8799fd8799fd8799fd8799f581caafb1196434cb837fd6f21323ca37b302dff6387e8a84b3fa28faf56ffd8799fd8799fd8799f581c52563c5410bff6a0d43ccebb7c37e1f69f5eb260552521adff33b9c2ffffffffd87a80ffffff";
        let data = hex::decode(hex_pool_datum).unwrap();
        let plutus_data = PlutusData::decode_fragment(&data).unwrap();
        let token_pair = TokenPair::try_from(&plutus_data).unwrap();

        let key = token_pair.key();
        assert_eq!(true, key.is_some());
        assert_eq!("29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6.4d494e:8db269c3ec630e06ae29f74bc39edd1f87c819f1056206e879a1cd61.446a65644d6963726f555344", key.unwrap());
    }

    #[test]
    fn test_invalid_key_for_ada_ada() {
        let token_pair = TokenPair {
            coin_a: PoolAsset::Ada,
            coin_b: PoolAsset::Ada,
        };
        let key = token_pair.key();
        assert_eq!(true, key.is_none());
    }
}
