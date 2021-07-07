mod dns;
mod tcp;
mod udp;
mod utils;
mod relay;

#[tokio::main]
async fn main() {
    let eps = vec![utils::Endpoint::new("127.0.0.1:15000", "localhost:20000")];
    relay::run(eps).await
}
