use pallas::ledger::primitives::babbage::PlutusData;

use super::model::{PoolAsset, TokenPair};

pub struct WingriderPoolDatum {
    pub coin_a: PoolAsset,
    pub coin_b: PoolAsset,
}

impl TryFrom<&PlutusData> for WingriderPoolDatum {
    type Error = ();

    fn try_from(value: &PlutusData) -> Result<Self, Self::Error> {
        if let PlutusData::Constr(pd) = value {
            if let Some(PlutusData::Constr(nested_pd)) = pd.fields.get(1) {
                let token_pair_pd = nested_pd.fields.get(0).ok_or(())?;
                let token_pair = TokenPair::try_from(token_pair_pd)?;
                return Ok(Self {
                    coin_a: token_pair.coin_a,
                    coin_b: token_pair.coin_b,
                });
            }
        }

        Err(())
    }
}

#[cfg(test)]
mod test {
    use pallas::ledger::primitives::{babbage::PlutusData, Fragment};

    use crate::reducers::liquidity_by_token_pair::{
        model::PoolAsset, utils::pool_asset_from, wingriders::WingriderPoolDatum,
    };

    #[test]
    fn test_decoding_pool_datum_ada_wrt() {
        let hex_pool_datum = "d8799f581c86ae9eebd8b97944a45201e4aec1330a72291af2d071644bba015959d8799fd8799fd8799f4040ffd8799f581cc0ee29a85b13209423b10447d3c2e6a50641a15c57770e27cb9d50734a57696e67526964657273ffff1b0000018511326ee01a027e81f01a07ea3059ffff";
        let data = hex::decode(hex_pool_datum).unwrap();
        let plutus_data = PlutusData::decode_fragment(&data).unwrap();
        let pool_datum = WingriderPoolDatum::try_from(&plutus_data).unwrap();
        assert_eq!(PoolAsset::Ada, pool_datum.coin_a);

        let wingrider_token = pool_asset_from(
            &String::from("c0ee29a85b13209423b10447d3c2e6a50641a15c57770e27cb9d5073"),
            &String::from("57696e67526964657273"),
        )
        .unwrap();
        assert_eq!(wingrider_token, pool_datum.coin_b);
    }
}
