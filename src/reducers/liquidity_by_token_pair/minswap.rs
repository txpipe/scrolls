use super::model::TokenPair;

pub type MinSwapPoolDatum = TokenPair;

#[cfg(test)]
mod test {
    use crate::reducers::liquidity_by_token_pair::minswap::MinSwapPoolDatum;
    use crate::reducers::liquidity_by_token_pair::model::PoolAsset;
    use crate::reducers::liquidity_by_token_pair::utils::pool_asset_from;

    use pallas::ledger::primitives::babbage::PlutusData;
    use pallas::ledger::primitives::Fragment;

    #[test]
    fn test_decoding_pool_datum_ada_min() {
        let hex_pool_datum = "d8799fd8799f4040ffd8799f581c29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6434d494eff1b00004ce6fb73282200d87a80ff";
        let data = hex::decode(hex_pool_datum).unwrap();
        let plutus_data = PlutusData::decode_fragment(&data).unwrap();
        let pool_datum = MinSwapPoolDatum::try_from(&plutus_data).unwrap();
        assert_eq!(PoolAsset::Ada, pool_datum.coin_a);
        let minswap_token = pool_asset_from(
            &String::from("29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6"),
            &String::from("4d494e"),
        )
        .unwrap();
        assert_eq!(minswap_token, pool_datum.coin_b);
    }

    #[test]
    fn test_decoding_pool_datum_min_djed() {
        let hex_pool_datum = "d8799fd8799f581c29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6434d494effd8799f581c8db269c3ec630e06ae29f74bc39edd1f87c819f1056206e879a1cd614c446a65644d6963726f555344ff1b000000012d9b96321b000000012dc40542d8799fd8799fd8799fd8799f581caafb1196434cb837fd6f21323ca37b302dff6387e8a84b3fa28faf56ffd8799fd8799fd8799f581c52563c5410bff6a0d43ccebb7c37e1f69f5eb260552521adff33b9c2ffffffffd87a80ffffff";
        let data = hex::decode(hex_pool_datum).unwrap();
        let plutus_data = PlutusData::decode_fragment(&data).unwrap();
        let pool_datum = MinSwapPoolDatum::try_from(&plutus_data).unwrap();
        let minswap_token = pool_asset_from(
            &String::from("29d222ce763455e3d7a09a665ce554f00ac89d2e99a1a83d267170c6"),
            &String::from("4d494e"),
        )
        .unwrap();
        let djed_token = pool_asset_from(
            &String::from("8db269c3ec630e06ae29f74bc39edd1f87c819f1056206e879a1cd61"),
            &String::from("446a65644d6963726f555344"),
        )
        .unwrap();
        assert_eq!(minswap_token, pool_datum.coin_a);
        assert_eq!(djed_token, pool_datum.coin_b);
    }
}
