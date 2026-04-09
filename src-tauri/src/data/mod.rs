#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use crate::errors::RetiscopeError;

use reticulum::hash::AddressHash;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

pub mod database;

#[allow(dead_code)]
#[derive(Clone, Serialize)]
pub struct AnnounceData {
    pub hops: u8,

    #[serde(serialize_with = "serialize_opt_hash")]
    pub transport_node: Option<AddressHash>,

    #[serde(serialize_with = "serialize_hash")]
    pub destination: AddressHash,

    #[serde(serialize_with = "serialize_hash")]
    pub iface: AddressHash,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StoredAnnounce {
    pub id: String, // or a newtype if you want to be strict
    pub hops: u8,
    #[serde(
        serialize_with = "serialize_opt_hash",
        deserialize_with = "deserialize_opt_hash"
    )]
    pub transport_node: Option<AddressHash>,
    #[serde(
        serialize_with = "serialize_hash",
        deserialize_with = "deserialize_hash"
    )]
    pub destination: AddressHash,
    #[serde(
        serialize_with = "serialize_hash",
        deserialize_with = "deserialize_hash"
    )]
    pub iface: AddressHash,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

fn serialize_hash<S>(hash: &AddressHash, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(&hash.to_hex_string())
}

fn serialize_opt_hash<S>(opt: &Option<AddressHash>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt {
        Some(h) => ser.serialize_str(&h.to_hex_string()),
        None => ser.serialize_none(),
    }
}

fn deserialize_opt_hash<'de, D>(de: D) -> Result<Option<AddressHash>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptHashVisitor;
    impl<'de> de::Visitor<'de> for OptHashVisitor {
        type Value = Option<AddressHash>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an optional hex string")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            AddressHash::new_from_hex_string(v)
                .map(Some)
                .map_err(|_| RetiscopeError::FailedToParse)
                .map_err(de::Error::custom)
        }
        fn visit_some<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
        where
            D2: Deserializer<'de>,
        {
            deserializer.deserialize_str(self)
        }
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }
    de.deserialize_any(OptHashVisitor) // deserialize_any instead of deserialize_option
}

fn deserialize_hash<'de, D>(de: D) -> Result<AddressHash, D::Error>
where
    D: Deserializer<'de>,
{
    struct HashVisitor;
    impl<'de> de::Visitor<'de> for HashVisitor {
        type Value = AddressHash;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a hex string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            AddressHash::new_from_hex_string(v)
                .map_err(|_| RetiscopeError::FailedToParse)
                .map_err(de::Error::custom)
        }
    }

    de.deserialize_str(HashVisitor)
}
