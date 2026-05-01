#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

#[allow(unused_imports)]
use crate::errors::RetiscopeError;

use reticulum::hash::AddressHash;

#[allow(unused_imports)]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::core::serde_helpers::{serialize_hash, serialize_opt_hash};

/// Raw announcement captured from the network.
/// Keeps only the minimal data needed for high‑throughput ingestion.
#[allow(dead_code)]
#[derive(Clone, Serialize)]
pub struct AnnounceData {
    /// Number of hops the packet travelled.
    pub hops: u8,

    /// Optional transport node that forwarded the packet.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_opt_hash"
    )]
    pub transport_node: Option<AddressHash>,

    /// Destination of the announcement.
    #[serde(serialize_with = "serialize_hash")]
    pub destination: AddressHash,

    /// Interface that received the announcement.
    #[serde(serialize_with = "serialize_hash")]
    pub iface: AddressHash,
}
