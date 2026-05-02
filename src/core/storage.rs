#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

#[allow(unused_imports)]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

// use futures::channel::mpsc;
use reticulum::hash::AddressHash;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};

use crate::core::AnnounceData;
use crate::core::serde_helpers::{
    deserialize_hash, deserialize_opt_hash, serialize_hash, serialize_opt_hash,
};
use crate::db::RetiscopeDB;

/// Persisted announcement stored in the database.
/// Includes capture metadata (`id`, `timestamp`) and deserialisation helpers.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct StoredAnnounce {
    /// Auto‑generated database identifier.
    pub id: String,

    /// Number of hops the packet travelled.
    pub hops: u8,

    /// Optional transport node that forwarded the packet.
    #[serde(
        default,
        deserialize_with = "deserialize_opt_hash",
        serialize_with = "serialize_opt_hash"
    )]
    pub transport_node: Option<AddressHash>,

    /// Destination of the announcement.
    #[serde(
        deserialize_with = "deserialize_hash",
        serialize_with = "serialize_hash"
    )]
    pub destination: AddressHash,

    /// Interface that generated the announcement.
    #[serde(
        deserialize_with = "deserialize_hash",
        serialize_with = "serialize_hash"
    )]
    pub iface: AddressHash,

    /// Timestamp of when the announcement was first captured.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Minimal node record kept in the database.
/// Stores the first and last time the node was observed.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct StoredNode {
    /// Auto‑generated database identifier.
    pub id: String,

    /// When the node was first seen.
    pub first_seen: chrono::DateTime<chrono::Utc>,

    /// Most recent time the node was observed.
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

pub async fn spawn_batcher(db: Arc<dyn RetiscopeDB>, mut rx: mpsc::Receiver<AnnounceData>) {
    tokio::spawn(async move {
        let mut batch = Vec::new();
        let mut timer = interval(Duration::from_secs(5));
        loop {
            tokio::select! {
                Some(data) = rx.recv() => {
                    batch.push(data);
                    if batch.len() >= 1000 {
                        let _ = db.save_announces(&mut batch).await;
                    }
                }
                _ = timer.tick() => {
                    if !batch.is_empty() {
                        let _ = db.save_announces(&mut batch).await;
                    }
                }
            }
        }
    });
}
