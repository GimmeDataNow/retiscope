use log::{self, info, trace};

use rand_core::OsRng;
use reticulum::destination::{DestinationName, SingleInputDestination};
use reticulum::hash::AddressHash;
use reticulum::identity::PrivateIdentity;
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};

use reticulum::iface::tcp_server::TcpServer;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time;

use surrealdb::engine::remote::ws::{Client, Ws, Wss};
use surrealdb::opt::auth::Root;
use surrealdb::{self, Surreal};
use surrealdb_types::{RecordId, SurrealValue};

use serde::{Deserialize, Serialize};

pub async fn db_init() -> Surreal<Client> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();
    db.signin(Root {
        username: "a".into(),
        password: "a".into(),
    })
    .await
    .unwrap();
    db.use_ns("main").use_db("main").await.unwrap();
    {
        let _ = db
            .query(
                r#"
                    DEFINE TABLE path_table SCHEMAFULL;

                    DEFINE FIELD hops          ON TABLE path_table TYPE int;
                    DEFINE FIELD iface         ON TABLE path_table TYPE option<string>;
                    DEFINE FIELD received_from ON TABLE path_table TYPE string;
                "#,
            )
            .await
            .unwrap();
    }
    db
}

pub fn db_serve() {}

#[derive(Debug, Serialize, Deserialize, SurrealValue, Clone)]
pub struct PathEntryWrapper {
    id: RecordId,
    hops: u8,
    received_from: Option<String>,
    iface: String,
}

pub async fn sync_path_table(
    db: &Surreal<Client>,
    entries: Vec<PathEntryWrapper>,
) -> surrealdb::Result<()> {
    db.query(
        r#"
        INSERT INTO path_table $data 
        ON DUPLICATE KEY UPDATE 
            content = $after;
    "#,
    )
    .bind(("data", entries))
    .await?;

    Ok(())
}

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

pub async fn add_transport_routes<P>(transport: &Transport, path: P)
where
    P: AsRef<std::path::Path>,
{
    log::info!("Add transport Nodes");

    let config_str = std::fs::read_to_string(path).unwrap();

    let config: Config = toml::from_str(&config_str).expect("Failed to parse TOML");

    let mut counter = 0;

    for iface in config.interfaces {
        if !iface.enabled {
            continue;
        }

        match iface.iface_type.as_str() {
            "TCPClientInterface" => {
                let host = iface.target_host.expect("Missing target_host");
                let port = iface.target_port.expect("Missing target_port");

                let _ = transport.iface_manager().lock().await.spawn(
                    TcpClient::new(format!("{}:{}", host, port)),
                    TcpClient::spawn,
                );
                counter += 1;
            }

            other => {
                log::warn!("Unsupported interface type: {}", other);
            }
        }
    }
    log::info!("{} interfaces started!", counter);
}

pub async fn router_init() {
    log::info!(">>> Reticulum Router + Path Monitor <<<");

    log::info!("DB init");
    let db = db_init().await;

    // Init transport
    let transport = Transport::new(TransportConfig::new(
        "router-node",
        &PrivateIdentity::new_from_rand(OsRng),
        true,
    ));

    // Start TCP Server Interface
    let _ = transport.iface_manager().lock().await.spawn(
        TcpServer::new("0.0.0.0:4242", transport.iface_manager()),
        TcpServer::spawn,
    );

    {
        add_transport_routes(
            &transport,
            "/home/hallow/.local/share/retiscope/connections.toml",
        )
        .await;
    }

    let mut path_rx = transport.subscribe_path_table();

    tokio::spawn(async move {
        // let mut local_path_map: HashMap<String, PathEntryWrapper> = HashMap::new();
        // Buffer to hold updates until the next sync interval
        let mut pending_updates: HashMap<String, PathEntryWrapper> = HashMap::new();
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));

        loop {
            tokio::select! {
                // 1. Collect incoming events
                res = path_rx.recv() => {
                    match res {
                        Ok(event) => {
                            let hash = event.destination.to_hex_string();
                            let new_entry = PathEntryWrapper {
                                id: RecordId::parse_simple(&format!("path_table:{}", hash)).unwrap(),
                                received_from: event.received_from.map(|a| a.to_hex_string()),
                                hops: event.hops,
                                iface: event.iface.to_hex_string(),
                            };

                            // DEDUPLICATION: Only keep if it's a new destination OR has fewer hops
                            let should_update = pending_updates.get(&hash)
                                .map_or(true, |existing| new_entry.hops < existing.hops);

                            if should_update {
                                pending_updates.insert(hash, new_entry);
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => log::warn!("Lagged {} events", n),
                        Err(broadcast::error::RecvError::Closed) => return,
                    }
                }
                // 2. Periodic Batch Sync
                _ = interval.tick() => {
                    if !pending_updates.is_empty() {
                        log::info!("Syncing {} unique path updates to DB", pending_updates.len());
                        let data: Vec<PathEntryWrapper> = pending_updates.values().cloned().collect();

                        // for (hash, entry) in pending_updates.drain() {
                            // log::trace!("Updating path {} ({} hops)", hash, entry.hops);
                            // local_path_map.insert(hash, entry);
                        // }

                        // Optional: Trigger your db_sync here with the full local_path_map
                        // let entries: Vec<_> = local_path_map.values().cloned().collect();
                        let _ = sync_path_table(&db, data).await;
                    }
                }
            }
        }
    });

    let _ = tokio::signal::ctrl_c().await;
    log::info!("Shutting down...");
}
