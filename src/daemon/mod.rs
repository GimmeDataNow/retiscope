use tracing::Instrument;
#[allow(unused_imports)]
use tracing::{debug, error, info, info_span, instrument, trace, warn};

use tokio::sync::mpsc;
use tokio::time::{Duration, interval};

use rand_core::OsRng;
use reticulum::hash::AddressHash;
use reticulum::identity::PrivateIdentity;
use reticulum::transport::{Transport, TransportConfig};

use crate::core::AnnounceData;
use crate::db::config::DatabaseConfig;
use crate::network::config::add_transport_routes;
use crate::paths::AppPaths;

#[allow(dead_code)]
#[instrument]
pub async fn run() {
    info!("Listener started");
    let app_paths = AppPaths::init();

    // database
    let db = DatabaseConfig::load_database_config(app_paths.get_database_config_path())
        .create_db()
        .await
        .inspect_err(|e| error!(error = %e, "Failed to connect to the database"))
        .expect("Failed to connect to the database");

    // this is needed
    let _ = db.init_db().await;

    // configure the transport
    let mut transport_config = TransportConfig::new(
        "reticulum-daemon",
        &PrivateIdentity::new_from_rand(OsRng),
        false,
    );
    transport_config.set_restart_outlinks(true);

    // transport
    let transport = Transport::new(transport_config);
    let _ = add_transport_routes(&transport, app_paths.get_connections_path()).await;

    let (tx, mut rx) = mpsc::channel::<AnnounceData>(100);

    let db_clone = db.clone();
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
        }
        // .instrument(tracing::Span::current()),
        .instrument(batcher_span),
    );

    // This task is to demonstrate the live updates from the database
    tokio::spawn(async move {
        let mut a = db_clone
            .watch_announces()
            .await
            .expect("watch_announces() failed");
        while let Ok(_data) = a.recv().await {
            // info!(watch_return = ?data, "watch_announces");
        }
        let mut b = db_clone.watch_nodes().await.expect("watch_nodes() failed");
        while let Ok(data) = b.recv().await {
            info!(watch_return = ?data, "watch_nodes");
        }

        // let c = db_clone
        //     .fetch_announces()
        //     .await
        //     .expect("fetch_announces() failed");

        // info!(payload = ?c, "");

        // let d = db_clone
        //     .fetch_nodes()
        //     .await
        //     .expect("fetch_announces() failed");
        // info!(payload = ?d, "");

        // warn!("tasks have finished")
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

        trace!(
            hops = data.hops,
            iface = format_args!("<{}>", data.iface.to_hex_string()),
            transport_node = format_args!(
                "<{}>",
                data.transport_node
                    .map(|h: AddressHash| h.to_hex_string())
                    .unwrap_or_else(|| "Self".to_string())
            ),
            destination = format_args!("<{}>", data.destination.to_hex_string()),
            "Announce Trace"
        );

        // send to batcher (non-blocking)
        let _ = tx.send(data).await;
    }
}
