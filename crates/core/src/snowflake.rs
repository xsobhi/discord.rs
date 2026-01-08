use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Snowflake(pub u64);

impl Snowflake {
    pub fn new(id: u64) -> Self {
        Snowflake(id)
    }
}

impl fmt::Display for Snowflake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Custom serialization to string
impl Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

// Custom deserialization from string or number (Discord sometimes sends numbers, but usually strings for IDs)
impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let id = u64::from_str(&s).map_err(serde::de::Error::custom)?;
        Ok(Snowflake(id))
    }
}

impl From<u64> for Snowflake {
    fn from(id: u64) -> Self {
        Snowflake(id)
    }
}

impl From<Snowflake> for u64 {
    fn from(s: Snowflake) -> Self {
        s.0
    }
}
