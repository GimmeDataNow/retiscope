use log::{self};

use rand_core::OsRng;
use reticulum::identity::PrivateIdentity;
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};
use std::sync::Arc;
use tokio::sync::{broadcast, Semaphore};

use reticulum::iface::tcp_server::TcpServer;
use std::collections::HashMap;

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{self, Surreal};
use surrealdb_types::{RecordId, SurrealValue};

use serde::{Deserialize, Serialize};

// pub async fn db_init() -> Surreal<Client> {
//     let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();
//     db.signin(Root {
//         username: "a".into(),
//         password: "a".into(),
//     })
//     .await
//     .unwrap();
//     db.use_ns("main").use_db("main").await.unwrap();
//     {
//         let _ = db
//             .query(
//                 r#"
//                     DEFINE TABLE path_table SCHEMAFULL;

//                     DEFINE FIELD hops          ON TABLE path_table TYPE int;
//                     DEFINE FIELD iface         ON TABLE path_table TYPE string;
//                     DEFINE FIELD received_from ON TABLE path_table TYPE option<string>;
//                     DEFINE FIELD last_seen     ON TABLE path_table TYPE datetime DEFAULT time::now();
//                 "#,
//             )
//             .await
//             .unwrap();
//     }
//     db
// }

pub fn db_serve() {}

#[derive(Debug, Serialize, Deserialize, SurrealValue, Clone)]
pub struct PathEntryWrapper {
    id: RecordId,
    hops: u8,
    received_from: Option<String>,
    iface: String,
}

// pub async fn sync_path_table(
//     db: &Surreal<Client>,
//     entries: Vec<PathEntryWrapper>,
// ) -> surrealdb::Result<()> {
//     let a = db
//         .query(
//             r#"
//         -- Insert only if record doesn't exist yet
//         INSERT IGNORE INTO path_table $data;

//         -- Update existing records only if conditions are met
//         FOR $entry IN $data {
//             UPDATE $entry.id
//             SET
//                 hops          = $entry.hops,
//                 iface         = $entry.iface,
//                 received_from = $entry.received_from,
//                 last_seen     = time::now()
//             WHERE
//                 (time::now() - last_seen) > 1h
//                 OR $entry.hops <= hops;
//         };
//     "#,
//         )
//         .bind(("data", entries))
//         .await?
//         .check();
//     match a {
//         Ok(_) => {}
//         Err(e) => {
//             log::error!("{:?}", e);
//         }
//     }

//     Ok(())
// }

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

    // log::info!("DB init");
    // let db = db_init().await;

    // Init transport
    let transport = Transport::new(TransportConfig::new(
        "router-node",
        &PrivateIdentity::new_from_rand(OsRng),
        true,
    ));

    // this is just plain bad but I will have to redo this later regardless
    add_transport_routes(
        &transport,
        "/home/hallow/.local/share/retiscope/connections.toml",
    )
    .await;

    // take care of the all the messages
    // ERROR: fucking deadlock again
    let mut announce_rx = transport.recv_announces().await;
    let semaphore = Arc::new(Semaphore::new(16)); // Slightly higher for 29 interfaces

    log::info!("Announce processor active and decoupled from Handler lock.");

    loop {
        // 2. We are now calling .recv() on the broadcast channel directly.
        // This DOES NOT trigger transport.handler.lock().
        match announce_rx.recv().await {
            Ok(event) => {
                let sem = semaphore.clone();
                tokio::spawn(async move {
                    let _permit = sem.acquire_owned().await.unwrap();

                    let dest = event.packet.destination;
                    log::trace!("Received announce for: {}", dest);

                    // DO NOT try to lock transport.handler here unless absolutely necessary.
                    // If you must, ensure it is a short-lived lock.
                });
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                log::warn!("Lagged by {} messages", n);
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }

    // loop {
    //     match annouce_fetch.recv().await {
    //         Ok(event) => {
    //             let sem = semaphore.clone();
    //             // let permit = semaphore.clone().acquire_owned().await.unwrap();

    //             tokio::spawn(async move {
    //                 let permit = sem.clone().acquire_owned().await.unwrap();
    //                 let dest = event.packet.destination;
    //                 let hops = event.packet.header.hops;
    //                 log::trace!("Received an announce {} hops away with: {}", hops, dest);

    //                 // process_event(event).await;
    //                 drop(permit);
    //             });
    //         }
    //         Err(broadcast::error::RecvError::Lagged(n)) => {
    //             // may not be the best but this is what reticulum-rs provides
    //             eprintln!("Receiver lagged by {} messages. Skipping to catch up.", n);
    //         }
    //         Err(broadcast::error::RecvError::Closed) => {
    //             println!("Sender dropped. Shutting down worker.");
    //             break;
    //         }
    //     }
    // }

    let _ = tokio::signal::ctrl_c().await;
    log::info!("Shutting down...");
}
