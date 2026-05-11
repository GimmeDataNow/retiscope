#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*, reload};

use clap::Parser;

mod arguments;
mod core;
mod daemon;
mod db;
mod errors;
mod network;
mod paths;
mod ui;

use crate::db::DatabaseHandle;

#[tokio::main]
async fn main() {
    // logging
    let initial_filter = EnvFilter::new("retiscope=info,reticulum=warn,surrealdb=error");
    let (filter_layer, _reload_handle) = reload::Layer::new(initial_filter);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer().with_target(false)) // prints to stdout/stderr
        .init();

    let args = arguments::Args::parse();

    let cancel_token = tokio_util::sync::CancellationToken::new();

    match args.command {
        arguments::Commands::Daemon => {
            info!("Starting Retiscope Daemon");

            let db = DatabaseHandle::create_and_configure().await;
            let _stream = daemon::run(cancel_token.clone(), db).await;

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("Ctrl+C received, shutting down gracefully...");
                }
                _ = cancel_token.cancelled() => {
                    error!("Critical background task failed. Exiting...");
                }
            }
        }
        arguments::Commands::Gui => {
            info!("Starting Retiscope GUI...");

            let db = DatabaseHandle::create_and_configure().await;
            let bundle = daemon::run(cancel_token.clone(), db).await;

            ui::run(bundle);
            cancel_token.cancel();

            info!("GUI closed, cleaning up");
        }
        _ => {
            panic!("This feature has not been implemented yet")
        }
    }
}
