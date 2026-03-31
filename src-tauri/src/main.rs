// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, reload, EnvFilter};

use clap::Parser;

use crate::files::save_identity;

mod arguments;
mod cli;
pub mod database;
pub mod errors;
pub mod files;

// #[derive(Parser)]
// #[command(name = "retiscope")]
// #[command(about = "A Reticulum Network Explorer", long_about = None)]
// #[command(version)]
// struct Args {
//     /// Use CLI
//     #[arg(long)]
//     cli: bool,
// }

#[tokio::main]
async fn main() {
    // logging
    let initial_filter = EnvFilter::new("retiscope=info,reticulum=warn,surrealdb=error");
    let (filter_layer, _reload_handle) = reload::Layer::new(initial_filter);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer().with_target(false)) // prints to stdout/stderr
        .init();

    // // args
    // let args = Args::parse();
    // if args.cli {
    //     cli::router().await;
    // } else {
    //     info!("gui started");
    //     retiscope_lib::run()
    // }
    // args
    let args = arguments::Args::parse();

    match args.command {
        arguments::Commands::Daemon => {
            info!("Starting daemon...");
            // run_daemon();
            // cli::router().await;
            cli::daemon::run().await;
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
