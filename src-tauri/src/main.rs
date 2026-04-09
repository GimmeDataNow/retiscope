// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[allow(unused_imports)]
use tracing::{debug, error, info, instrument, trace, warn};

use tracing_subscriber::{fmt, prelude::*, reload, EnvFilter};

use clap::Parser;

mod arguments;

#[tokio::main]
async fn main() {
    // logging
    let initial_filter = EnvFilter::new("retiscope=debug,reticulum=info,surrealdb=error");
    let (filter_layer, _reload_handle) = reload::Layer::new(initial_filter);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer().with_target(false)) // prints to stdout/stderr
        .init();

    let args = arguments::Args::parse();

    match args.command {
        arguments::Commands::Daemon => {
            info!("Starting daemon...");
            retiscope_lib::cli::listener::run().await;
        }
        arguments::Commands::Service => {
            info!("Starting service...");
            // run_service();
        }
        arguments::Commands::Database => {
            info!("Starting database...");
            // run_service();
        }
        #[cfg(feature = "gui")]
        arguments::Commands::Gui => {
            info!("Launching GUI...");
            // tauri::Builder::default().run(tauri::generate_context!()).expect("error while running tauri application");
            retiscope_lib::run();
        }
    }
}
