#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

#[allow(unused_imports)]
use crate::errors::RetiscopeError;

// use futures::SinkExt;
// use futures::channel::mpsc;
use reticulum::hash::AddressHash;
use reticulum::transport::Transport;
use tokio::sync::mpsc;

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

pub struct CaptureEngine {
    pub transport: Transport,
    pub live_tx: tokio::sync::broadcast::Sender<AnnounceData>,
}

impl CaptureEngine {
    pub fn new(transport: Transport) -> Self {
        let (live_tx, _) = tokio::sync::broadcast::channel(1000);
        Self { transport, live_tx }
    }

    pub async fn run(&self, db_tx: mpsc::Sender<AnnounceData>) {
        let mut announce_receiver = self.transport.recv_announces().await;

        while let Ok(ok) = announce_receiver.recv().await {
            // let data = AnnounceData::from_reticulum(ok); // Move formatting logic to a helper
            let data = AnnounceData {
                hops: ok.packet.header.hops,
                transport_node: ok.packet.transport,
                destination: ok.packet.destination,
                iface: ok.iface,
            };

            // Send to DB Batcher
            let _ = db_tx.send(data.clone()).await;

            // Broadcast to GUI (ignores error if no one is listening)
            let _ = self.live_tx.send(data);
        }
    }
}
