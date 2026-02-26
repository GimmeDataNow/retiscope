// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread::sleep, time};

use clap::Parser;
use log::{self, info};

mod cli;

#[derive(Parser)]
#[command(name = "retiscope")]
#[command(about = "A Reticulum Network Explorer", long_about = None)]
#[command(version)]
struct Args {
    /// Launch in CLI mode instead of GUI
    #[arg(long)]
    cli: bool,
}

fn main() {
    let args = Args::parse();
    if args.cli {
        info!("CLI started");
        cli::cli_init();
        // loop {
        //     sleep(time::Duration::from_secs(1));
        //     info!("wow");
        // }
    } else {
        info!("GUI started");
        retiscope_lib::run()
    }
}
