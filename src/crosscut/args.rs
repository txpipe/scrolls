use core::fmt;
use std::{ops::Deref, str::FromStr};

use pallas::network::miniprotocols::{Point, MAINNET_MAGIC, TESTNET_MAGIC};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

/// A serialization-friendly chain Point struct using a hex-encoded hash
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointArg(pub u64, pub String);

impl TryInto<Point> for PointArg {
    type Error = crate::Error;

    fn try_into(self) -> Result<Point, Self::Error> {
        let hash = hex::decode(&self.1)
            .map_err(|_| Self::Error::message("can't decode point hash hex value"))?;

        Ok(Point::Specific(self.0, hash))
    }
}

impl FromStr for PointArg {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains(',') {
            let mut parts: Vec<_> = s.split(',').collect();
            let slot = parts
                .remove(0)
                .parse()
                .map_err(|_| Self::Err::message("can't parse slot number"))?;

            let hash = parts.remove(0).to_owned();
            Ok(PointArg(slot, hash))
        } else {
            Err(Self::Err::message(
                "Can't parse chain point value, expecting `slot,hex-hash` format",
            ))
        }
    }
}

impl ToString for PointArg {
    fn to_string(&self) -> String {
        format!("{},{}", self.0, self.1)
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

pub(crate) fn deserialize_magic_arg<'de, D>(deserializer: D) -> Result<Option<MagicArg>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MagicArgVisitor;

    impl<'de> Visitor<'de> for MagicArgVisitor {
        type Value = Option<MagicArg>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<MagicArg>, E>
        where
            E: serde::de::Error,
        {
            let value = FromStr::from_str(value).map_err(serde::de::Error::custom)?;
            Ok(Some(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Option<MagicArg>, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(MagicArg(value)))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(MagicArgVisitor)
}
