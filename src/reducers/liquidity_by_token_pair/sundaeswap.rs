use core::f32;

use pallas::ledger::primitives::babbage::PlutusData;

use super::model::{PoolAsset, TokenPair};

#[derive(Debug, PartialEq)]
pub struct SundaePoolDatum {
    pub coin_a: PoolAsset,
    pub coin_b: PoolAsset,
    pub fee: f32,
}

impl TryFrom<&PlutusData> for SundaePoolDatum {
    type Error = ();

    fn try_from(value: &PlutusData) -> Result<Self, Self::Error> {
        if let PlutusData::Constr(pd) = value {
            let token_pair_pd = pd.fields.get(0).ok_or(())?;
            let token_pair = TokenPair::try_from(token_pair_pd)?;

            if let Some(PlutusData::Constr(fee_pd)) = pd.fields.get(3) {
                return match (fee_pd.fields.get(0), fee_pd.fields.get(1)) {
                    (
                        Some(PlutusData::BigInt(pallas::ledger::primitives::babbage::BigInt::Int(
                            numerator,
                        ))),
                        Some(PlutusData::BigInt(pallas::ledger::primitives::babbage::BigInt::Int(
                            denominator,
                        ))),
                    ) => {
                        let n = i32::try_from(i128::from(*numerator)).ok().ok_or(())?;
                        let d = i32::try_from(i128::from(*denominator)).ok().ok_or(())?;
                        Ok(Self {
                            coin_a: token_pair.coin_a,
                            coin_b: token_pair.coin_b,
                            fee: (n as f32) / (d as f32),
                        })
                    }
                    _ => Err(()),
                };
            }
        }

        Err(())
    }
}

impl std::fmt::Display for SundaePoolDatum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SundaeTokenPair {{ coin_a: ({:?}), coin_b: ({:?}), fee: {:?} }}",
            self.coin_a, self.coin_b, self.fee
        )
    }
}

#[cfg(test)]
mod test {
    use pallas::ledger::primitives::{babbage::PlutusData, Fragment};

    use crate::reducers::liquidity_by_token_pair::{
        model::PoolAsset, sundaeswap::SundaePoolDatum, utils::pool_asset_from,
    };

    #[test]
    fn test_decoding_pool_datum_ada_sun() {
        let hex_pool_datum = "d8799fd8799fd8799f4040ffd8799f581c9a9693a9a37912a5097918f97918d15240c92ab729a0b7c4aa144d774653554e444145ffff41081b0000105a99e0fa59d8799f031903e8ffff";
        let data = hex::decode(hex_pool_datum).unwrap();
        let plutus_data = PlutusData::decode_fragment(&data).unwrap();
        let pool_datum = SundaePoolDatum::try_from(&plutus_data).unwrap();
        assert_eq!(PoolAsset::Ada, pool_datum.coin_a);

        let sundae_token = pool_asset_from(
            &String::from("9a9693a9a37912a5097918f97918d15240c92ab729a0b7c4aa144d77"),
            &String::from("53554e444145"),
        )
        .unwrap();
        assert_eq!(sundae_token, pool_datum.coin_b);
        assert_eq!(f32::from(0.003), pool_datum.fee);
    }
}
