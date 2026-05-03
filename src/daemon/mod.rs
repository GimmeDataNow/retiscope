use tracing::Instrument;
#[allow(unused_imports)]
use tracing::{debug, error, info, info_span, instrument, trace, warn};

use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, interval};
use tokio_util::sync::CancellationToken;

use rand_core::OsRng;
use reticulum::identity::PrivateIdentity;
use reticulum::transport::{Transport, TransportConfig};

use crate::core::AnnounceData;
use crate::db::config::DatabaseConfig;
use crate::network::config::add_transport_routes;
use crate::paths::AppPaths;

pub struct StreamBundle {
    pub transport: Transport,
    // pub raw_packets: broadcast::Receiver<reticulum::packet::Packet>,
    pub announces: broadcast::Sender<AnnounceData>,
}

#[allow(dead_code)]
#[instrument]
pub async fn run(cancel: CancellationToken) -> StreamBundle {
    info!("Daemon started");
    let app_paths = AppPaths::init();

    // db
    let db = DatabaseConfig::load_database_config(app_paths.get_database_config_path())
        .create_db()
        .await
        .expect("Failed to connect to the database");

    let _ = db.init_db().await;

    // transport
    let mut transport_config = TransportConfig::new(
        "reticulum-daemon",
        &PrivateIdentity::new_from_rand(OsRng),
        false,
    );
    transport_config.set_restart_outlinks(true);

    let transport = Transport::new(transport_config);
    let _ = add_transport_routes(&transport, app_paths.get_connections_path()).await;

    // internal channels
    let (tx, mut rx) = mpsc::channel::<AnnounceData>(100);

    // returned (external) channels
    let (ext_tx, _) = broadcast::channel::<AnnounceData>(1024);

    // batcher task
    let db_clone = db.clone();
    let batcher_span = info_span!("batcher_task");
    tokio::spawn(
        async move {
            let mut batch = Vec::new();
            let mut timer = interval(Duration::from_secs(5));
            loop {
                tokio::select! {
                    Some(data) = rx.recv() => {
                        batch.push(data);
                        if batch.len() >= 10 {
                            let _ = db_clone.save_announces(&mut batch).await;
                        }
                    }
                    _ = timer.tick() => {
                        if !batch.is_empty() {
                            let _ = db_clone.save_announces(&mut batch).await;
                        }
                    }
                }
            }
        }
        .instrument(batcher_span),
    );

    // recv and pass data forward
    let mut announce_receiver = transport.recv_announces().await;
    let mut a = transport.iface_rx();

    // temp
    tokio::spawn(async move {
        while let Ok(ok) = a.recv().await {
            // ok.packet.;
            info!(hops = %ok.packet.header.hops, destination = ok.packet.destination.to_hex_string() ,"packet");
        }
        warn!("Announce ifac loop exited!");
    });

    let cloned_channel = ext_tx.clone();
    tokio::spawn(async move {
        info!("Starting Announce Monitor...");

        // Use a loop to keep the task alive even if the receiver errors out initially
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;

            while let Ok(ok) = announce_receiver.recv().await {
                let data = AnnounceData {
                    hops: ok.packet.header.hops,
                    transport_node: ok.packet.transport,
                    destination: ok.packet.destination,
                    iface: ok.iface,
                };

                let _ = cloned_channel.send(data);
            }

            // If we reach here, the receiver died.
            // Check if we should quit or try to reconnect.
            if cancel.is_cancelled() {
                break;
            }

            warn!("Network receiver disconnected. Retrying in 10s...");
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });

    StreamBundle {
        transport,
        announces: ext_tx,
    }
}
