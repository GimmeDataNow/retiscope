use log::{self, info};

use rand_core::OsRng;
use reticulum::destination::{DestinationName, SingleInputDestination};
use reticulum::identity::PrivateIdentity;
use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};

pub fn cli_init() {
    info!("cli function start");

    let transport = Transport::new(TransportConfig::new(
        "server",
        &PrivateIdentity::new_from_rand(OsRng),
        true,
    ));
}
