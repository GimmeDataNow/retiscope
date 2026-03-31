use std::path::PathBuf;

use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, instrument, trace, warn};

use rand_core::OsRng;
use reticulum::identity::PrivateIdentity;
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};

use surrealdb::{self, Surreal};

// use crate::database;
use crate::database::data::AnnounceData;
use crate::errors::RetiscopeError;
use crate::files::get_paths;
use crate::{database, files};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Config {
    interfaces: Vec<InterfaceConfig>,
}

#[derive(Debug, Deserialize)]
struct InterfaceConfig {
    #[serde(rename = "type")]
    iface_type: String,
    enabled: bool,
    target_host: Option<String>,
    target_port: Option<u16>,
}

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

#[allow(dead_code)]
#[instrument]
pub async fn run() {
    info!("Daemon started");
    // database
    let db_config_path = get_paths().config.join("database.toml");
    let db_config = database::load_database_config(db_config_path);
    let db = db_config.create_db().await.unwrap();
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

        debug!(
            hops = data.hops,
            iface = format_args!("<{}>", data.iface.to_hex_string()),
            transport_node = format_args!(
                "<{}>",
                data.transport_node
                    .map(|h| h.to_hex_string())
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
