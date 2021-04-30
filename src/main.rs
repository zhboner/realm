use tokio;
mod relay;
mod resolver;
mod udp;

#[tokio::main]
async fn main() {
    let relay_configs = realm::parse_arguments();
    relay::start_relay(relay_configs).await;
}
