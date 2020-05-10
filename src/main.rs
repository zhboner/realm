use std::net;
use tokio;
mod relay;

#[tokio::main]
async fn main() {
    let cli = realm::parse_arguments();
    let client_socket: net::SocketAddr = cli.client.parse().expect("Unable to parse client address");
    let remote_socket: net::SocketAddr = cli.remote.parse().expect("Unable to parse remote address");

    relay::start(client_socket, remote_socket).await;
}
