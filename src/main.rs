// use std::net;
use tokio;

mod relay;
mod resolver;

#[tokio::main]
async fn main() {
    let relay_config = realm::parse_arguments();
    relay::start_relay(relay_config).await;
}
