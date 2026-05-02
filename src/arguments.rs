use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "retiscope")]
#[command(about = "A Reticulum Network Visualizer")]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch the background collector
    Daemon,
    /// Run the database for the daemon
    Database,
    /// Run a node with services
    Service,

    // #[cfg(feature = "gui")]
    /// Launch the graphical interface
    Gui,
}
