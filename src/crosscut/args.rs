use pallas::network::miniprotocols::{
    Point,
    MAINNET_MAGIC,
    TESTNET_MAGIC,
    // PREVIEW_MAGIC, PRE_PRODUCTION_MAGIC,
};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, str::FromStr};

use crate::Error;

// TODO: use from pallas once available
pub const PRE_PRODUCTION_MAGIC: u64 = 1;
pub const PREVIEW_MAGIC: u64 = 2;

/// A serialization-friendly chain Point struct using a hex-encoded hash
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PointArg {
    Origin,
    Specific(u64, String),
}

impl TryInto<Point> for PointArg {
    type Error = crate::Error;

    fn try_into(self) -> Result<Point, Self::Error> {
        match self {
            PointArg::Origin => Ok(Point::Origin),
            PointArg::Specific(slot, hash_hex) => {
                let hash = hex::decode(&hash_hex)
                    .map_err(|_| Self::Error::message("can't decode point hash hex value"))?;

                Ok(Point::Specific(slot, hash))
            }
        }
    }
}

impl From<Point> for PointArg {
    fn from(other: Point) -> Self {
        match other {
            Point::Origin => PointArg::Origin,
            Point::Specific(slot, hash) => PointArg::Specific(slot, hex::encode(hash)),
        }
    }
}

impl FromStr for PointArg {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            x if s.contains(',') => {
                let mut parts: Vec<_> = x.split(',').collect();
                let slot = parts
                    .remove(0)
                    .parse()
                    .map_err(|_| Self::Err::message("can't parse slot number"))?;

                let hash = parts.remove(0).to_owned();
                Ok(PointArg::Specific(slot, hash))
            }
            "origin" => Ok(PointArg::Origin),
            _ => Err(Self::Err::message(
                "Can't parse chain point value, expecting `slot,hex-hash` format",
            )),
        }
    }
}

impl ToString for PointArg {
    fn to_string(&self) -> String {
        match self {
            PointArg::Origin => "origin".to_string(),
            PointArg::Specific(slot, hash) => format!("{},{}", slot, hash),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MagicArg(pub u64);

impl Deref for MagicArg {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for MagicArg {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let m = match s {
            "testnet" => MagicArg(TESTNET_MAGIC),
            "mainnet" => MagicArg(MAINNET_MAGIC),
            "preview" => MagicArg(PREVIEW_MAGIC),
            "preprod" => MagicArg(PRE_PRODUCTION_MAGIC),
            _ => MagicArg(u64::from_str(s).map_err(|_| "can't parse magic value")?),
        };

        Ok(m)
    }
}

impl Default for MagicArg {
    fn default() -> Self {
        Self(MAINNET_MAGIC)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum IntersectConfig {
    Tip,
    Origin,
    Point(u64, String),
    Fallbacks(Vec<(u64, String)>),
}

impl IntersectConfig {
    pub fn get_point(&self) -> Option<Point> {
        match self {
            IntersectConfig::Point(slot, hash) => {
                let hash = hex::decode(hash).expect("valid hex hash");
                Some(Point::Specific(*slot, hash))
            }
            _ => None,
        }
    }

    pub fn get_fallbacks(&self) -> Option<Vec<Point>> {
        match self {
            IntersectConfig::Fallbacks(all) => {
                let mapped = all
                    .iter()
                    .map(|(slot, hash)| {
                        let hash = hex::decode(hash).expect("valid hex hash");
                        Point::Specific(*slot, hash)
                    })
                    .collect();

                Some(mapped)
            }
            _ => None,
        }
    }
}

/// Optional configuration to stop processing new blocks after processing:
///   1. a block with the given hash
///   2. the first block on or after a given absolute slot
///   3. TODO: a total of X blocks 
#[derive(Deserialize, Debug, Clone)]
pub struct FinalizeConfig {
    until_hash: Option<String>,
    max_block_slot: Option<u64>,
    // max_block_quantity: Option<u64>,
}

pub fn should_finalize(
    config: &Option<FinalizeConfig>,
    last_point: &Point,
    // block_count: u64,
) -> bool {
    let config = match config {
        Some(x) => x,
        None => return false,
    };

    if let Some(expected) = &config.until_hash {
        if let Point::Specific(_, current) = last_point {
            return expected == &hex::encode(current);
        }
    }
    
    if let Some(max) = config.max_block_slot {
        if last_point.slot_or_default() >= max {
            return true;
        }
    }
    
    // if let Some(max) = config.max_block_quantity {
    //     if block_count >= max {
    //         return true;
    //     }
    // }

    false
}

/// Well-known information about the blockhain network
///
/// Some of the logic in Scrolls depends on particular characteristic of the
/// network that it's consuming from. For example: time calculation and bech32
/// encoding. This struct groups all of these blockchain network specific
/// values.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChainWellKnownInfo {
    pub magic: u64,
    pub byron_epoch_length: u32,
    pub byron_slot_length: u32,
    pub byron_known_slot: u64,
    pub byron_known_hash: String,
    pub byron_known_time: u64,
    pub shelley_epoch_length: u32,
    pub shelley_slot_length: u32,
    pub shelley_known_slot: u64,
    pub shelley_known_hash: String,
    pub shelley_known_time: u64,
    pub address_network_id: u8,
    pub adahandle_policy: String,
}

impl ChainWellKnownInfo {
    /// Hardcoded values for mainnet
    pub fn mainnet() -> Self {
        ChainWellKnownInfo {
            magic: MAINNET_MAGIC,
            byron_epoch_length: 432000,
            byron_slot_length: 20,
            byron_known_slot: 0,
            byron_known_time: 1506203091,
            byron_known_hash: "f0f7892b5c333cffc4b3c4344de48af4cc63f55e44936196f365a9ef2244134f"
                .to_string(),
            shelley_epoch_length: 432000,
            shelley_slot_length: 1,
            shelley_known_slot: 4492800,
            shelley_known_hash: "aa83acbf5904c0edfe4d79b3689d3d00fcfc553cf360fd2229b98d464c28e9de"
                .to_string(),
            shelley_known_time: 1596059091,
            address_network_id: 1,
            adahandle_policy: "f0ff48bbb7bbe9d59a40f1ce90e9e9d0ff5002ec48f232b49ca0fb9a"
                .to_string(),
        }
    }

    /// Hardcoded values for testnet
    pub fn testnet() -> Self {
        ChainWellKnownInfo {
            magic: TESTNET_MAGIC,
            byron_epoch_length: 432000,
            byron_slot_length: 20,
            byron_known_slot: 0,
            byron_known_time: 1564010416,
            byron_known_hash: "8f8602837f7c6f8b8867dd1cbc1842cf51a27eaed2c70ef48325d00f8efb320f"
                .to_string(),
            shelley_epoch_length: 432000,
            shelley_slot_length: 1,
            shelley_known_slot: 1598400,
            shelley_known_hash: "02b1c561715da9e540411123a6135ee319b02f60b9a11a603d3305556c04329f"
                .to_string(),
            shelley_known_time: 1595967616,
            address_network_id: 0,
            adahandle_policy: "8d18d786e92776c824607fd8e193ec535c79dc61ea2405ddf3b09fe3"
                .to_string(),
        }
    }

    pub fn preview() -> Self {
        ChainWellKnownInfo {
            magic: PREVIEW_MAGIC,
            byron_epoch_length: 432000,
            byron_slot_length: 20,
            byron_known_slot: 0,
            byron_known_hash: "".to_string(),
            byron_known_time: 1660003200,
            shelley_epoch_length: 432000,
            shelley_slot_length: 1,
            shelley_known_slot: 25260,
            shelley_known_hash: "cac921895ef5f2e85f7e6e6b51b663ab81b3605cd47d6b6d66e8e785e5c65011"
                .to_string(),
            shelley_known_time: 1660003200,
            address_network_id: 0,
            adahandle_policy: "".to_string(),
        }
    }

    /// Hardcoded values for the "pre-prod" testnet
    pub fn preprod() -> Self {
        ChainWellKnownInfo {
            magic: PRE_PRODUCTION_MAGIC,
            byron_epoch_length: 432000,
            byron_slot_length: 20,
            byron_known_slot: 0,
            byron_known_hash: "9ad7ff320c9cf74e0f5ee78d22a85ce42bb0a487d0506bf60cfb5a91ea4497d2"
                .to_string(),
            byron_known_time: 1654041600,
            shelley_epoch_length: 432000,
            shelley_slot_length: 1,
            shelley_known_slot: 86400,
            shelley_known_hash: "c4a1595c5cc7a31eda9e544986fe9387af4e3491afe0ca9a80714f01951bbd5c"
                .to_string(),
            shelley_known_time: 1654041600,
            address_network_id: 0,
            adahandle_policy: "".to_string(),
        }
    }

    /// Uses the value of the magic to return either mainnet or testnet
    /// hardcoded values.
    pub fn try_from_magic(magic: u64) -> Result<ChainWellKnownInfo, Error> {
        match magic {
            MAINNET_MAGIC => Ok(Self::mainnet()),
            TESTNET_MAGIC => Ok(Self::testnet()),
            PREVIEW_MAGIC => Ok(Self::preview()),
            PRE_PRODUCTION_MAGIC => Ok(Self::preprod()),
            _ => Err(Error::ConfigError(
                "can't infer well-known chain infro from specified magic".into(),
            )),
        }
    }
}

impl Default for ChainWellKnownInfo {
    fn default() -> Self {
        Self::mainnet()
    }
}
