#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use crate::errors::RetiscopeError;

use async_trait::async_trait;

use reticulum::hash::AddressHash;
use serde::Serialize;

#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

pub mod database;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnnounceData {
    pub hops: u8,
    pub transport_node: Option<AddressHash>,
    pub destination: AddressHash,
    pub iface: AddressHash,
}

#[derive(Serialize)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct DBAnnounce {
    hops: u8,
    transport_node: Option<String>,
    destination: String,
    iface: String,
}

impl From<AnnounceData> for DBAnnounce {
    fn from(data: AnnounceData) -> Self {
        Self {
            hops: data.hops,
            transport_node: data.transport_node.map(|h| h.to_hex_string()),
            destination: data.destination.to_hex_string(),
            iface: data.iface.to_hex_string(),
        }
    }
}
