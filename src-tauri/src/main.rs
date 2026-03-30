// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, reload, EnvFilter};

use clap::Parser;

use crate::files::save_identity;

mod cli;
pub mod errors;
pub mod files;

#[derive(Parser)]
#[command(name = "retiscope")]
#[command(about = "A Reticulum Network Explorer", long_about = None)]
#[command(version)]
struct Args {
    /// Use CLI
    #[arg(long)]
    cli: bool,
}

#[tokio::main]
async fn main() {
    // logging
    let initial_filter = EnvFilter::new("retiscope=info,reticulum=warn,surrealdb=error");
    let (filter_layer, _reload_handle) = reload::Layer::new(initial_filter);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer().with_target(false)) // prints to stdout/stderr
        .init();
    // let _ = save_identity("identity".as_bytes()).inspect_err(|e| error!(error = e.to_string()));

    // args
    let args = Args::parse();
    if args.cli {
        cli::router().await;
    } else {
        info!("gui started");
        retiscope_lib::run()
    }
}
