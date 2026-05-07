#[allow(unused_imports)]
use tracing::{debug, error, info, info_span, instrument, trace, warn};

use crate::errors::RetiscopeError;

use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::Transport;
use serde::Deserialize;
use std::path::PathBuf;

/// Configuration for the network transport layer.
///
/// This structure represents the `connections.toml` file used to define
/// how Retiscope connects to the wider Reticulum network.
#[derive(Debug, Deserialize)]
struct Config {
    /// A collection of individual interface definitions.
    interfaces: Vec<InterfaceConfig>,
}

/// Parameters for a specific Reticulum interface.
///
/// Maps directly to the TOML configuration. Currently supports `TCPClientInterface`.
#[derive(Debug, Deserialize)]
struct InterfaceConfig {
    /// The type of interface (e.g., "TCPClientInterface").
    #[serde(rename = "type")]
    iface_type: String,
    /// Whether this interface should be initialized on startup.
    enabled: bool,
    /// The remote IP or hostname for TCP-based connections.
    target_host: Option<String>,
    /// The remote port for TCP-based connections.
    target_port: Option<u16>,
}

/// Dynamically initializes network routes based on a configuration file.
///
/// This function parses the provided TOML file and spawns active interfaces
/// into the Reticulum [`Transport`] instance.
///
/// # Arguments
/// * `transport` - A reference to the initialized Reticulum transport layer.
/// * `path` - The filesystem path to the `connections.toml` file.
///
/// # Errors
/// Returns [`RetiscopeError::FailedToParse`] if the config is malformed.
#[instrument(skip(transport, path))]
pub async fn add_transport_routes(
    transport: &Transport,
    path: PathBuf,
) -> Result<(), RetiscopeError> {
    info!(path = path.to_str(), "adding transport nodes");

    let config_str = std::fs::read_to_string(path).unwrap();

    let config: Config = toml::from_str(&config_str)
        .inspect_err(|_| error!("failed to parse connections"))
        .map_err(|_| RetiscopeError::FailedToParse)?;

    let (mut started, mut skipped) = (0, 0);

    for iface in config.interfaces {
        if !iface.enabled {
            skipped += 1;
            continue;
        }

        match iface.iface_type.as_str() {
            "TCPClientInterface" => {
                let host = iface.target_host.expect("Missing target_host");
                let port = iface.target_port.expect("Missing target_port");

                let a = transport.iface_manager().lock().await.spawn(
                    TcpClient::new(format!("{}:{}", host, port)),
                    TcpClient::spawn,
                );
                info!("new iface: <{}> @ {}:{}", a.to_hex_string(), host, port);
                started += 1;
            }

            other => {
                skipped += 1;
                warn!("Unsupported interface type: {}", other);
            }
        }
    }
    info!(started = started, skipped = skipped, "Interfaces started!");
    Ok(())
}
