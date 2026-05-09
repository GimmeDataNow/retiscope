use tracing::Instrument;
#[allow(unused_imports)]
use tracing::{debug, error, info, info_span, instrument, trace, warn};

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, interval};
use tokio_util::sync::CancellationToken;

use rand_core::OsRng;
use reticulum::identity::PrivateIdentity;
use reticulum::iface::RxMessage;
use reticulum::transport::{Transport, TransportConfig};

use crate::core::AnnounceData;
use crate::db::config::DatabaseConfig;
use crate::network::config::add_transport_routes;
use crate::paths::AppPaths;

#[allow(dead_code)]
pub struct StreamBundle {
    pub raw_interface_packets: broadcast::Sender<RxMessage>,
    pub announces: broadcast::Sender<AnnounceData>,
}

#[allow(dead_code)]
#[instrument(skip(cancel))]
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

    let transport = Arc::new(Transport::new(transport_config));
    let _ = add_transport_routes(&transport, app_paths.get_connections_path()).await;

    // channels
    let (tx, mut rx) = mpsc::channel::<AnnounceData>(100); // channel for the db sync
    let (ext_tx, _) = broadcast::channel::<AnnounceData>(1024); // returned (external) channels
    let (packet_tx, _keep_alive_rx) = broadcast::channel::<RxMessage>(1024); // packet channel

    // batcher task
    let batcher_span = info_span!("batcher_task");
    tokio::spawn(
        async move {
            let mut batch = Vec::new();
            let mut timer = interval(Duration::from_secs(5));
            loop {
                tokio::select! {
                    Some(data) = rx.recv() => {
                        batch.push(data);
                        if batch.len() >= 100 {
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
        }
        .instrument(batcher_span),
    );

    // raw packets
    let raw_packets_task = info_span!("raw_packets_task");
    let t_clone_1 = transport.clone();
    let packet_tx_clone = packet_tx.clone();
    let cancel_clone = cancel.clone();
    tokio::spawn(
        async move {
            let mut iface_rx_msgs = t_clone_1.iface_rx();

            loop {
                while let Ok(ok) = iface_rx_msgs.recv().await {
                    debug!(hops = %ok.packet.header.hops, destination = ok.packet.destination.to_hex_string(), transport = ok.packet.transport.map(|a| a.to_hex_string()) ,"packet");
                    let _ = packet_tx_clone
                        .send(ok)
                        .inspect_err(|e| error!(error = %e, "Failed to send packet"));
                }
                if cancel_clone.is_cancelled() {
                    break;
                }

                warn!("Network receiver disconnected. Retrying in 10s...");
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            warn!("Announce ifac loop exited!");
        }
        .instrument(raw_packets_task),
    );

    let announce_task = info_span!("announce_task");
    let cloned_channel = ext_tx.clone();
    let mut announce_receiver = transport.recv_announces().await;

    tokio::spawn(
        async move {
            info!("Starting Announce Monitor");

            loop {
                while let Ok(ok) = announce_receiver.recv().await {
                    let data = AnnounceData {
                        hops: ok.packet.header.hops,
                        transport_node: ok.packet.transport,
                        destination: ok.packet.destination,
                        iface: ok.iface,
                    };

                    let _ = tx.send(data.clone()).await;
                    let _ = cloned_channel.send(data);
                }

                if cancel.is_cancelled() {
                    break;
                }

                warn!("Network receiver disconnected. Retrying in 10s...");
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
        .instrument(announce_task),
    );

    StreamBundle {
        announces: ext_tx,
        raw_interface_packets: packet_tx,
    }
}
