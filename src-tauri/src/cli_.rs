use std::path::PathBuf;

use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, instrument, trace, warn};

use rand_core::OsRng;
use reticulum::hash::AddressHash;
use reticulum::identity::PrivateIdentity;
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{self, Surreal};
use surrealdb_types::SurrealValue;

use crate::errors::RetiscopeError;
use crate::files;
use serde::{Deserialize, Serialize};

pub async fn db_init() -> Surreal<Client> {
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();
    db.signin(Root {
        username: "a".into(),
        password: "a".into(),
    })
    .await
    .unwrap();
    // db.select("announce").live()
    db.use_ns("main").use_db("main").await.unwrap();
    {
        let _ = db
            .query(
                r#"
                    -- 1. Unique Nodes
                    DEFINE TABLE node SCHEMAFULL;
                    DEFINE FIELD first_seen ON TABLE node TYPE datetime DEFAULT time::now();
                    DEFINE FIELD last_seen  ON TABLE node TYPE datetime DEFAULT time::now();
                    DEFINE INDEX node_addr  ON TABLE node COLUMNS id UNIQUE;
                    
                    -- 2. Announce Events (The Log)
                    DEFINE TABLE announce SCHEMAFULL;
                    DEFINE FIELD destination    ON TABLE announce TYPE record<node>;
                    DEFINE FIELD transport_node ON TABLE announce TYPE option<record<node>>;
                    DEFINE FIELD iface          ON TABLE announce TYPE string;
                    DEFINE FIELD hops           ON TABLE announce TYPE int;
                    DEFINE FIELD timestamp      ON TABLE announce TYPE datetime DEFAULT time::now();
                    
                    -- Index for fast "Give me history for Node X" queries
                    DEFINE INDEX announce_dest ON TABLE announce COLUMNS destination;

                    -- For the "last_seen" logic in the UPSERT
                    DEFINE INDEX idx_node_id ON TABLE node COLUMNS id UNIQUE;
                    
                    -- For the "history" and deduplication check
                    -- This makes 'WHERE destination = ... ORDER BY timestamp' instant
                    DEFINE INDEX idx_announce_lookup ON TABLE announce COLUMNS destination, timestamp;

                "#,
            )
            .await
            .unwrap();
    }
    db
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
#[derive(Debug, Clone)]
pub struct AnnounceData {
    pub hops: u8,
    pub transport_node: Option<AddressHash>,
    pub destination: AddressHash,
    pub iface: AddressHash,
}

#[derive(Serialize, SurrealValue)]
struct DbAnnounce {
    hops: u8,
    transport_node: Option<String>,
    destination: String,
    iface: String,
}

impl From<AnnounceData> for DbAnnounce {
    fn from(data: AnnounceData) -> Self {
        Self {
            hops: data.hops,
            transport_node: data.transport_node.map(|h| h.to_hex_string()),
            destination: data.destination.to_hex_string(),
            iface: data.iface.to_hex_string(),
        }
    }
}

#[allow(dead_code)]
#[instrument]
pub async fn router() {
    info!("router started");
    let db = db_init().await;

    // Init transport
    let transport = Transport::new(TransportConfig::new(
        "router-node",
        &PrivateIdentity::new_from_rand(OsRng),
        false,
    ));

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
                        flush_to_db(&db, &mut batch).await;
                    }
                }
                _ = timer.tick() => {
                    if !batch.is_empty() {
                        flush_to_db(&db, &mut batch).await;
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

async fn flush_to_db(db: &Surreal<Client>, batch: &mut Vec<AnnounceData>) {
    if batch.is_empty() {
        debug!("Announce Entries are empty. Nothing to give to the db");
        return;
    }

    let original_data = std::mem::take(batch);

    // par_iter might be interesting here, not sure of the performance benefits tho
    let data_to_send: Vec<DbAnnounce> = original_data.into_iter().map(DbAnnounce::from).collect();

    // this was created in large part by gemini
    // seems alright tho
    let query = r#"
        FOR $entry IN $data {
            -- 1. Heartbeat for the Destination Node
            UPSERT type::record("node", $entry.destination) 
            SET last_seen = time::now();

            -- 2. Heartbeat for the Relay Node (if it exists)
            IF $entry.transport_node != NONE {
                UPSERT type::record("node", $entry.transport_node) 
                SET last_seen = time::now();
            };

            -- 3. Smart Announce Logic
            LET $dest_id = type::record("node", $entry.destination);
            LET $relay_id = IF $entry.transport_node != NONE { 
                type::record("node", $entry.transport_node) 
            } ELSE { 
                NONE 
            };

            LET $last = (
                SELECT id, hops, transport_node, timestamp
                FROM announce 
                WHERE destination = $dest_id 
                ORDER BY timestamp DESC 
                LIMIT 1
            )[0];

            IF !$last OR $last.hops != $entry.hops OR $last.transport_node != $relay_id {
                CREATE announce SET
                    destination = $dest_id,
                    transport_node = $relay_id,
                    hops = $entry.hops,
                    iface = $entry.iface,
                    timestamp = time::now();
            } ELSE {
                UPDATE $last.id SET timestamp = time::now();
            };
        };
    "#;

    match db.query(query).bind(("data", data_to_send)).await {
        Ok(response) => {
            // check for errors in the response
            if let Err(e) = response.check() {
                error!(error = %e, "Batch query execution failed");
            } else {
                trace!("Batch sync complete");
            }
        }
        Err(e) => error!(error = %e, "Failed to send batch query"),
    }
    batch.clear();
}
