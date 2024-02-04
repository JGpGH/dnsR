use anyhow::Result;
use std::time::Duration;

use clap::Parser;
use handler::Handler;
use options::Options;
use trust_dns_server::ServerFuture;
use tokio::net::{TcpListener, UdpSocket};

mod handler;
mod options;
mod provision;
mod ip_addr_serde;

const TCP_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> Result<()> {
    let options = Options::parse();
    let network = provision::get_network()?;
    println!("using: {:?}, serving zone {:?}", options, network.zone);
    let handler = Handler::from_network(network);

    // create DNS server
    let mut server = ServerFuture::new(handler);

    // register UDP listeners
    for udp in &options.udp {
        server.register_socket(UdpSocket::bind(udp).await?);
    }

    // register TCP listeners
    for tcp in &options.tcp {
        server.register_listener(TcpListener::bind(&tcp).await?, TCP_TIMEOUT);
    }

    println!("prepared, binding to: {} and {}", options.udp[0], options.tcp[0]);
    // run DNS server
    server.block_until_done().await?;

    Ok(())
}
