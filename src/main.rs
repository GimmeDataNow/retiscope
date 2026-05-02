#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*, reload};

use clap::Parser;
use gpui::*;
use gpui_component::*;

mod arguments;
mod core;
mod daemon;
mod db;
mod errors;
mod network;
mod paths;
mod ui;

#[tokio::main]
async fn main() {
    // logging
    let initial_filter = EnvFilter::new("retiscope=debug,reticulum=warn,surrealdb=error");
    let (filter_layer, _reload_handle) = reload::Layer::new(initial_filter);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer().with_target(false)) // prints to stdout/stderr
        .init();

    let args = arguments::Args::parse();

    match args.command {
        arguments::Commands::Daemon => {
            let _ = daemon::run().await;
        }
        arguments::Commands::Gui => {
            let _ = ui::run();
        }
        _ => {
            panic!("This feature has not been implemented yet")
        }
    }
}

// pub async fn run() -> tokio::sync::broadcast::Sender<AnnounceData> {
//     // Get Paths
//     let app_paths = AppPaths::init();

//     // Setup DB
//     let db = DatabaseConfig::load_database_config(app_paths.get_database_config_path())
//         .create_db()
//         .await
//         .unwrap();

//     // Setup Transport
//     let transport = setup_transport();
//     let _ = add_transport_routes(&transport, app_paths.get_connections_path()).await;

//     // Initialize Engine
//     let engine = CaptureEngine::new(transport);
//     let (db_tx, db_rx) = mpsc::channel(100);

//     // Start Tasks
//     spawn_batcher(db, db_rx).await;

//     let live_tx = engine.live_tx.clone();
//     tokio::spawn(async move {
//         engine.run(db_tx).await;
//     });

//     live_tx
// }
