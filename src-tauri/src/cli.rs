use log::{self, info, trace};

use rand_core::OsRng;
use reticulum::destination::{DestinationName, SingleInputDestination};
use reticulum::identity::PrivateIdentity;
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};

use reticulum::iface::tcp_server::TcpServer;
use std::time::Duration;
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
        // db.query("DEFINE FIELD timestamp ON TABLE path_table;")
        //     .await
        //     .unwrap();
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

#[derive(Debug, Serialize, Deserialize, SurrealValue)]
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
    // Because PathEntryWrapper now implements SurrealValue,
    // Vec<PathEntryWrapper> automatically satisfies the bound for .bind()
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

pub async fn router_init() {
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

    // 3. Spawn the Monitoring Task
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            // Acquire the lock to the TransportHandler
            let handler = handler_link.lock().await;

            let path_map = handler.path_table_map_ref();

            if path_map.is_empty() {
                log::info!("No paths discovered yet.");
            } else {
                let entries: Vec<PathEntryWrapper> = path_map
                    .iter()
                    .map(|(hash, entry)| {
                        PathEntryWrapper {
                            // Construct the record ID manually: "table_name:identifier"
                            id: RecordId::parse_simple(&format!(
                                "path_table:{}",
                                hash.to_hex_string()
                            ))
                            .unwrap(),
                            received_from: entry.received_from.clone().to_hex_string(),
                            hops: entry.hops,
                            iface: entry.iface.clone().to_hex_string(),
                        }
                    })
                    .collect();
                let a = sync_path_table(&db, entries).await;
                log::error!("{:?}", a);
            }

            // Drop the lock explicitly if you have more work in the loop
            drop(handler);
        }
    });

    log::info!("Router is active. Monitoring PathTable every 5s.");

    let _ = tokio::signal::ctrl_c().await;
    log::info!("Shutting down...");
}
