use futures::future::try_join;
use futures::FutureExt;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;
use tokio;
use tokio::io;
use tokio::net;

use crate::resolver;
use realm::RelayConfig;

pub async fn start_relay(config: RelayConfig) {
    let (send, recv) = mpsc::channel::<std::net::IpAddr>();
    let remote_addr = config.remote_address.clone();
    thread::spawn(move || resolver::dns_resolve(remote_addr, send));
    run(config, recv).await.unwrap();
}

pub async fn run(
    config: RelayConfig,
    recv: mpsc::Receiver<std::net::IpAddr>,
) -> Result<(), Box<dyn Error>> {
    let client_socket: SocketAddr =
        format!("{}:{}", config.listening_address, config.listening_port)
            .parse()
            .unwrap();
    let mut listener = net::TcpListener::bind(&client_socket).await?;

    let mut remote_ip = recv.recv().unwrap();
    let mut remote_socket: SocketAddr = format!("{}:{}", remote_ip, config.remote_port)
        .parse()
        .unwrap();
    println!("Listening on: {}", client_socket);

    while let Ok((inbound, _)) = listener.accept().await {
        if let Ok(new_ip) = recv.try_recv() {
            remote_ip = new_ip;
            remote_socket = format!("{}:{}", remote_ip, config.remote_port)
                .parse()
                .unwrap();
        }
        let transfer = transfer(inbound, remote_socket.clone()).map(|r| {
            if let Err(_) = r {
                return;
            }
        });
        tokio::spawn(transfer);
    }
    Ok(())
}

async fn transfer(
    mut inbound: net::TcpStream,
    remote_socket: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut outbound = net::TcpStream::connect(remote_socket).await?;
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);

    try_join(client_to_server, server_to_client).await?;

    Ok(())
}
