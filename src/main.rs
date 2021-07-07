mod relay;

#[tokio::main]
async fn main() {
    let eps = vec![relay::Endpoint::new("127.0.0.1:15000", "localhost:20000")];
    relay::run(eps).await
}
