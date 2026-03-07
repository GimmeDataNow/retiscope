use log::{self};

use rand_core::OsRng;
use reticulum::identity::PrivateIdentity;
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};

use reticulum::iface::tcp_server::TcpServer;
use std::collections::HashMap;

use surrealdb::engine::remote::ws::{Client, Ws};
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
                    DEFINE FIELD iface         ON TABLE path_table TYPE string;
                    DEFINE FIELD received_from ON TABLE path_table TYPE option<string>;
                    DEFINE FIELD last_seen     ON TABLE path_table TYPE datetime DEFAULT time::now();
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
    let a = db
        .query(
            r#"
        -- Insert only if record doesn't exist yet
        INSERT IGNORE INTO path_table $data;

        -- Update existing records only if conditions are met
        FOR $entry IN $data {
            UPDATE $entry.id
            SET
                hops          = $entry.hops,
                iface         = $entry.iface,
                received_from = $entry.received_from,
                last_seen     = time::now()
            WHERE
                (time::now() - last_seen) > 1h
                OR $entry.hops <= hops;
        };
    "#,
        )
        .bind(("data", entries))
        .await?
        .check();
    match a {
        Ok(_) => {}
        Err(e) => {
            log::error!("{:?}", e);
        }
    }

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
    const MAX_MAP_SIZE: usize = 10000;

    tokio::spawn(async move {
        let mut local_path_map: HashMap<String, PathEntryWrapper> = HashMap::new();
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));

        loop {
            tokio::select! {
                res = path_rx.recv() => {
                    match res {
                        Ok(event) => {
                            let hash = event.destination.to_hex_string();

                            // 1. Better Path Check (Logic: Less hops = Better)
                            let is_better = if let Some(existing) = local_path_map.get(&hash) {
                                event.hops < existing.hops
                            } else {
                                // 2. Size Limit Check (Only add new entries if under limit)
                                if local_path_map.len() < MAX_MAP_SIZE {
                                    true // We have room, add it
                                } else {
                                    // ONLY log this when we actually hit the limit
                                    log::warn!("Memory map limit reached ({}). Skipping new discovery: {}", MAX_MAP_SIZE, hash);
                                    false
                                }
                            };

                            if is_better {
                                // log::trace!("better path found");
                                local_path_map.insert(hash.clone(), PathEntryWrapper {
                                    id: RecordId::parse_simple(&format!("path_table:{}", hash)).unwrap(),
                                    received_from: event.received_from.map(|a| a.to_hex_string()),
                                    hops: event.hops,
                                    iface: event.iface.to_hex_string(),
                                });
                            } else {
                                // log::warn!("The buffer for paths is full, skipping this event");
                            }

                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            log::warn!("Channel lagged: missed {} events", n);
                        }
                        Err(_) => return,
                    }
                }

                _ = interval.tick() => {
                    if !local_path_map.is_empty() {
                        // 3. Atomically take the map to clear memory immediately
                        let snapshot = std::mem::take(&mut local_path_map);

                        let db_clone = db.clone();

                        tokio::spawn(async move {
                            let entries: Vec<PathEntryWrapper> = snapshot.into_iter().map(|(_, v)| v).collect();
                            log::info!("DB sync started with {}", entries.len());
                            if let Err(e) = sync_path_table(&db_clone, entries).await {
                                log::error!("DB Sync failed: {}", e);
                            }
                        });
                    }

                }
            }
        }
    });

    let _ = tokio::signal::ctrl_c().await;
    log::info!("Shutting down...");
}
