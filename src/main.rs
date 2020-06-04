use tokio;
mod relay;
mod resolver;
use futures::future::join_all;

#[tokio::main]
async fn main() {
    let relay_configs = realm::parse_arguments();
    let mut runners = vec![];

    for config in relay_configs {
        runners.push(tokio::spawn(relay::start_relay(config)))
    }

    join_all(runners).await;
}
