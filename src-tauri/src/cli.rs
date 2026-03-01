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
                    DEFINE FIELD iface         ON TABLE path_table TYPE string;
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
    received_from: String,
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

pub fn add_transport_routes<P>(transport: Transport, path: P)
where
    P: AsRef<std::path::Path>,
{
    log::info!("Add transport Nodes");

    let config = std::fs::read_to_string(path);
}

pub async fn router_init() {
    log::info!(">>> Reticulum Router + Path Monitor <<<");

    log::info!("DB init");
    let db = db_init().await;

    // 1. Initialize Transport
    let transport = Transport::new(TransportConfig::new(
        "router-node",
        &PrivateIdentity::new_from_rand(OsRng),
        true,
    ));

    // TCP Server
    let _ = transport.iface_manager().lock().await.spawn(
        TcpServer::new("0.0.0.0:4242", transport.iface_manager()),
        TcpServer::spawn,
    );

    // TCP Clients (Hubs)
    let hub_1 = "202.61.243.41:4965";
    let hub_2 = "reticulum.betweentheborders.com:4242";
    let hub_3 = "dublin.connect.reticulum.network:4965";
    let hub_4 = "202.61.243.41:4965";
    let hub_5 = "193.26.158.230:4965";

    for target in [hub_1, hub_2, hub_3, hub_4, hub_5] {
        let addr = transport
            .iface_manager()
            .lock()
            .await
            .spawn(TcpClient::new(target), TcpClient::spawn);
        log::trace!(
            "Started Client interface {} to {}",
            addr.to_hex_string(),
            target
        );
    }

    let mut path_rx = transport.recv_path_events().await;
    log::warn!("monitoring: subscribed to path events");
    tokio::spawn(async move {
        log::warn!("monitoring: task started");
        loop {
            log::warn!("monitoring: waiting for event");
            match path_rx.recv().await {
                Ok(event) => log::warn!("monitoring: got event!"),
                Err(e) => log::warn!("monitoring: error {:?}", e),
            }
        }
    });

    log::info!("Router is active. Press Ctrl+C to shut down.");
    let _ = tokio::signal::ctrl_c().await;
    log::info!("Shutting down...");
}

pub async fn router_init_old() {
    log::info!(">>> Reticulum Router + Path Monitor <<<");

    log::info!("DB init");
    let db = db_init().await;

    // Init transport
    let transport = Transport::new(TransportConfig::new(
        "router-node",
        &PrivateIdentity::new_from_rand(OsRng),
        true, // Routing enabled
    ));
    // Get the Arc<Mutex<TransportHandler>> and clone it for the thread
    let handler_link = transport.get_handler();

    // Start TCP Server Interface
    let _ = transport.iface_manager().lock().await.spawn(
        TcpServer::new("0.0.0.0:4242", transport.iface_manager()),
        TcpServer::spawn,
    );

    let addr = transport
        .iface_manager()
        .lock()
        .await
        .spawn(TcpClient::new("202.61.243.41:4965"), TcpClient::spawn);
    log::trace!("Started Client interface {}", addr.to_hex_string());

    let addr = transport.iface_manager().lock().await.spawn(
        TcpClient::new("reticulum.betweentheborders.com:4242"),
        TcpClient::spawn,
    );
    log::trace!("Started Client interface {}", addr.to_hex_string());

    let mut path_rx = transport.subscribe_path_table();
    tokio::spawn(async move {
        let mut local_path_map: HashMap<String, PathEntryWrapper> = HashMap::new();
        loop {
            match path_rx.recv().await {
                Ok(event) => {
                    let hash = event.destination.to_hex_string();
                    local_path_map.insert(
                        hash.clone(),
                        PathEntryWrapper {
                            id: RecordId::parse_simple(&format!("path_table:{}", hash)).unwrap(),
                            received_from: event.received_from.map(|a| a.to_hex_string()),
                            hops: event.hops,
                            iface: event.iface.to_hex_string(),
                        },
                    );
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    log::warn!("Lagged, skipped {} path events", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    log::error!("Path event channel closed");
                    return;
                }
            }
        }
    });

    log::info!("Router is active. Monitoring PathTable every 5s.");

    let _ = tokio::signal::ctrl_c().await;
    log::info!("Shutting down...");
}
