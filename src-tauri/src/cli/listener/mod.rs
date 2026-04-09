//! The Network Ingestor and Telemetry Coordinator.
//!
//! This module serves as the primary bridge between the Reticulum Network and the
//! Retiscope database. It is responsible for initializing the transport layer,
//! managing network interfaces, and capturing `Announce` packets for long-term storage.
//!
//! # Architecture
//!
//! The ingestor operates using a **Split-Task Architecture**:
//!
//! 1. **Capture Loop**: The main `run` loop asynchronously receives raw announces
//!    from the Reticulum `Transport`. It performs lightweight formatting into
//!    [`AnnounceData`] and pushes them into an internal MPSC channel.
//!
//! 2. **Batcher Task**: An independent background task drains the channel and
//!    aggregates data into batches.
//!
//! # Features
//!
//! * **Interface Management**: Dynamically loads and spawns Reticulum interfaces
//!   (e.g., `TCPClientInterface`) based on a `connections.toml` configuration file.
//! * **Batching**: Flushes data to the database either when a
//!   threshold of 1000 records is met or when a 5-second "heartbeat" timer expires.
//!
//! # Future Expansion
//!
//! This module is designed to evolve into a **Management Nexus**, capable of
//! not just observing announces, but also issuing remote management commands
//! and monitoring service health across the network.
//!
//! # Errors
//!
//! Failures in interface spawning or configuration parsing are logged as
//! warnings or errors but generally do not halt the entire ingestion process.
#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

// OsRng is used for the creation of the identity
use rand_core::OsRng;
use reticulum::hash::AddressHash;
use reticulum::identity::PrivateIdentity;
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};

use crate::data::database::{self, RetiscopeDB};
use crate::data::AnnounceData;

use crate::errors::RetiscopeError;
use crate::files;

use serde::Deserialize;

/// Configuration for the network transport layer.
///
/// This structure represents the `connections.toml` file used to define
/// how Retiscope connects to the wider Reticulum network.
#[derive(Debug, Deserialize)]
struct Config {
    /// A collection of individual interface definitions.
    interfaces: Vec<InterfaceConfig>,
}

/// Parameters for a specific Reticulum interface.
///
/// Maps directly to the TOML configuration. Currently supports `TCPClientInterface`.
#[derive(Debug, Deserialize)]
struct InterfaceConfig {
    /// The type of interface (e.g., "TCPClientInterface").
    #[serde(rename = "type")]
    iface_type: String,
    /// Whether this interface should be initialized on startup.
    enabled: bool,
    /// The remote IP or hostname for TCP-based connections.
    target_host: Option<String>,
    /// The remote port for TCP-based connections.
    target_port: Option<u16>,
}

/// Dynamically initializes network routes based on a configuration file.
///
/// This function parses the provided TOML file and spawns active interfaces
/// into the Reticulum [`Transport`] instance.
///
/// # Arguments
/// * `transport` - A reference to the initialized Reticulum transport layer.
/// * `path` - The filesystem path to the `connections.toml` file.
///
/// # Errors
/// Returns [`RetiscopeError::FailedToParse`] if the config is malformed.
#[instrument(skip(transport, path))]
pub async fn add_transport_routes(
    transport: &Transport,
    path: PathBuf,
) -> Result<(), RetiscopeError> {
    info!(path = path.to_str(), "adding transport nodes");

    let config_str = std::fs::read_to_string(path).unwrap();

    let config: Config = toml::from_str(&config_str)
        .inspect_err(|_| error!("failed to parse connections"))
        .map_err(|_| RetiscopeError::FailedToParse)?;

    let (mut started, mut skipped) = (0, 0);

    for iface in config.interfaces {
        if !iface.enabled {
            skipped += 1;
            continue;
        }

        match iface.iface_type.as_str() {
            "TCPClientInterface" => {
                let host = iface.target_host.expect("Missing target_host");
                let port = iface.target_port.expect("Missing target_port");

                let a = transport.iface_manager().lock().await.spawn(
                    TcpClient::new(format!("{}:{}", host, port)),
                    TcpClient::spawn,
                );
                info!("new iface: <{}> @ {}:{}", a.to_hex_string(), host, port);
                started += 1;
            }

            other => {
                skipped += 1;
                warn!("Unsupported interface type: {}", other);
            }
        }
    }
    info!(started = started, skipped = skipped, "Interfaces started!");
    Ok(())
}

/// The primary entry point for the network observation and database ingestion service.
///
/// This function orchestrates the entire lifecycle of the ingestor:
/// 1. Initializes the database connection from `database.toml`.
/// 2. Configures and starts the Reticulum [`Transport`] layer.
/// 3. Establishes network interfaces via [`add_transport_routes`].
/// 4. Spawns a background **Batcher Task** to aggregate announces.
/// 5. Enters a high-frequency loop to capture and relay network telemetry.
///
/// # Batching Behavior
/// To optimize database performance, announces are not written immediately. Instead:
/// * They are flushed every **5 seconds** if any are pending.
/// * They are flushed immediately if the buffer reaches **1000 records**.
#[allow(dead_code)]
#[instrument]
pub async fn run() {
    info!("Listener started");
    // database
    let db_config_path = files::get_paths().config.join("database.toml");
    let db_config = database::load_database_config(db_config_path);
    let db: Arc<dyn RetiscopeDB> = db_config.create_db().await.unwrap();
    // this is needed
    let _ = db.init_db().await;

    // configure the transport
    let mut transport_config = TransportConfig::new(
        "reticulum-daemon",
        &PrivateIdentity::new_from_rand(OsRng),
        false,
    );
    transport_config.set_restart_outlinks(true);

    // init transport
    let transport = Transport::new(transport_config);

    // this is just plain bad but I will have to redo this later regardless
    // files
    let paths = files::get_paths();
    let path = paths.config.join("connections.toml");
    files::ensure_file(&path);

    let _ = add_transport_routes(
        &transport, // "/home/hallow/.local/share/retiscope/connections.toml",
        path,
    )
    .await;

    let (tx, mut rx) = mpsc::channel::<AnnounceData>(100);

    let db_clone = db.clone();
    // batcher task
    tokio::spawn(async move {
        let mut batch = Vec::new();
        let mut timer = interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                Some(data) = rx.recv() => {
                    batch.push(data);
                    // flush early if batch is huge
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

    // This task is to demonstrate the live updates from the database
    tokio::spawn(async move {
        let mut a = db_clone
            .watch_announces()
            .await
            .expect("watch_announces() failed");
        while let Ok(data) = a.recv().await {
            info!(watch_return = ?data, "watch_announces");
        }
    });

    // send announces to the batcher task
    let mut announce_receiver = transport.recv_announces().await;
    while let Ok(ok) = announce_receiver.recv().await {
        // format data
        let data = AnnounceData {
            hops: ok.packet.header.hops,
            transport_node: ok.packet.transport,
            destination: ok.packet.destination,
            iface: ok.iface,
        };

        trace!(
            hops = data.hops,
            iface = format_args!("<{}>", data.iface.to_hex_string()),
            transport_node = format_args!(
                "<{}>",
                data.transport_node
                    .map(|h: AddressHash| h.to_hex_string())
                    .unwrap_or_else(|| "Self".to_string())
            ),
            destination = format_args!("<{}>", data.destination.to_hex_string()),
            "Announce Trace"
        );

        // send to batcher (non-blocking)
        let _ = tx.send(data).await;
    }

    // let _ = tokio::signal::ctrl_c().await;
    // info!("Shutting down...");
}
