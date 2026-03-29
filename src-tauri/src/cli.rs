use tracing::{debug, error, info, instrument, span, trace, warn, Level};
// use log::{self};

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

#[instrument(skip(transport, path))]
pub async fn add_transport_routes<P>(transport: &Transport, path: P)
where
    P: AsRef<std::path::Path> + std::fmt::Debug,
{
    info!("adding transport nodes");

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

                let a = transport.iface_manager().lock().await.spawn(
                    TcpClient::new(format!("{}:{}", host, port)),
                    TcpClient::spawn,
                );
                info!("spawning an iface with: {}:{} @ {}", host, port, a);
                counter += 1;
            }

            other => {
                warn!("Unsupported interface type: {}", other);
            }
        }
    }
    info!("{} interfaces started!", counter);
}

#[instrument]
pub async fn router() {
    info!("router started");

    // Init transport
    let transport = Transport::new(TransportConfig::new(
        "router-node",
        &PrivateIdentity::new_from_rand(OsRng),
        false,
    ));

    // this is just plain bad but I will have to redo this later regardless
    add_transport_routes(
        &transport,
        "/home/hallow/.local/share/retiscope/connections.toml",
    )
    .await;

    // take care of the all the messages
    // ERROR: fucking deadlock again

    // let mut announce_rx = transport.recv_announces().await;
    // trace!("here");
    // let mut iface_rx_maybe = transport.iface_rx();

    // loop {
    //     // trace!("maybe here?");
    //     match iface_rx_maybe.recv().await {
    //         Ok(ok) => {
    //             // debug!("address: {}", ok.address);
    //             match ok.packet.header.packet_type {
    //                 reticulum::packet::PacketType::Announce => {
    //                     // info!("address: {}", ok.address)
    //                     // info!(
    //                     //     "dest: {}, distance: {}",
    //                     //     ok.packet.destination, ok.packet.header.hops
    //                     // )
    //                     info!(
    //                         iface = %ok.address.to_hex_string(),
    //                         destination = %ok.packet.destination.to_hex_string(),
    //                         via = ?ok.packet.transport.map(|h| h.to_hex_string()),
    //                         hops = ?ok.packet.header.hops, // field name may differ
    //                         "announce"
    //                     );
    //                 }
    //                 _ => {
    //                     debug!("discard")
    //                 }
    //             }
    //         }
    //         Err(broadcast::error::RecvError::Lagged(n)) => {
    //             warn!("Lagged behind by {} messages, continuing", n);
    //             // Don't break — just keep going
    //         }
    //         Err(e) => {
    //             error!("Fatal error: {}", e);
    //             break;
    //         }
    //     }
    // }
    let mut announce = transport.recv_announces().await;

    loop {
        match announce.recv().await {
            Ok(ok) => {
                trace!(
                    // distance
                    hops = ok.packet.header.hops,
                    // which node did it come from. If None then the node itself was the sender
                    transport_node = format_args!(
                        "<{}>",
                        ok.packet
                            .transport
                            .map(|h| h.to_hex_string())
                            .unwrap_or_else(|| "Self".to_string())
                    ),
                    // the actual destination
                    destination = format_args!("<{}>", ok.packet.destination.to_hex_string()),
                    // descriptor
                    "Announce Trace"
                );
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }

    // let _ = tokio::signal::ctrl_c().await;
    // info!("Shutting down...");
}
