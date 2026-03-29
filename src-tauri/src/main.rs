// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing::info;
use tracing_subscriber::{fmt, prelude::*, reload, EnvFilter};

use clap::Parser;

mod cli;

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
    let initial_filter = EnvFilter::new("retiscope=info,reticulum=warn,surrealdb=error");
    let (filter_layer, _reload_handle) = reload::Layer::new(initial_filter);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer().with_target(false)) // prints to stdout/stderr
        .init();
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let args = Args::parse();

    if args.cli {
        cli::router().await;
    } else {
        info!("GUI started");
        retiscope_lib::run()
    }
}
