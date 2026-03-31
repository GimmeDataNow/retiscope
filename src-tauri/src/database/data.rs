#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use crate::errors::RetiscopeError;

use async_trait::async_trait;

use reticulum::hash::AddressHash;
use serde::Serialize;

#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

#[async_trait]
pub trait RetiscopeDB: Send + Sync {
    /// Initializes the database schema and administrative users.
    ///
    /// This method ensures the database is in a ready state by:
    /// * Applying required table schemas and indexes.
    /// * Provisioning internal system users and permissions.
    ///
    /// # Errors
    ///
    /// TODO
    async fn set_up_db(&self) -> Result<(), RetiscopeError>;
    /// Prepares the database for use.
    ///
    /// This method ensures the correct database state by:
    /// * Logging into the correct user.
    /// * Selecting the right namespace and database.
    ///
    /// # Errors
    ///
    /// TODO
    async fn init_db(&self) -> Result<(), RetiscopeError>;

    /// Writes the announces to the database.
    ///
    /// This method writes this data by:
    /// * Upserting each announce
    /// * Upserting each node and the respective timestamps
    ///
    /// # Errors
    ///
    /// TODO
    async fn save_announces(&self, announce: &mut Vec<AnnounceData>) -> Result<(), RetiscopeError>;
}

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
