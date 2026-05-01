#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use crate::errors::RetiscopeError;

use reticulum::hash::AddressHash;

#[allow(unused_imports)]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[allow(dead_code)]
pub(crate) fn serialize_hash<S>(hash: &AddressHash, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(&hash.to_hex_string())
}

#[allow(dead_code)]
pub(crate) fn serialize_opt_hash<S>(opt: &Option<AddressHash>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match opt {
        Some(h) => ser.serialize_str(&h.to_hex_string()),
        None => ser.serialize_none(),
    }
}

#[allow(dead_code)]
pub(crate) fn deserialize_opt_hash<'de, D>(de: D) -> Result<Option<AddressHash>, D::Error>
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
            let hex = v.splitn(2, ':').last().unwrap_or(v);
            AddressHash::new_from_hex_string(hex)
                .map(Some)
                .map_err(|_| de::Error::custom(RetiscopeError::FailedToParse))
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

#[allow(dead_code)]
pub(crate) fn deserialize_hash<'de, D>(de: D) -> Result<AddressHash, D::Error>
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
            let hex = v.splitn(2, ':').last().unwrap_or(v);
            AddressHash::new_from_hex_string(hex)
                .map_err(|_| de::Error::custom(RetiscopeError::FailedToParse))
        }
    }

    de.deserialize_str(HashVisitor)
}
