use futures::future::join_all;
use futures::future::try_join;
use futures::FutureExt;
use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use tokio;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net;

use crate::resolver;
use crate::udp;
use realm::RelayConfig;

// Initialize DNS recolver
// Set up channel between listener and resolver

pub async fn start_relay(configs: Vec<RelayConfig>) {
    let default_ip: IpAddr = String::from("0.0.0.0").parse::<IpAddr>().unwrap();
    let remote_addrs: Vec<String> = configs.iter().map(|x| x.remote_address.clone()).collect();

    let mut remote_ips: Vec<Arc<RwLock<std::net::IpAddr>>> = Vec::new();
    for _ in 0..remote_addrs.len() {
        remote_ips.push(Arc::new(RwLock::new(default_ip.clone())))
    }
    let cloned_remote_ips = remote_ips.clone();

    tokio::spawn(resolver::resolve(remote_addrs, cloned_remote_ips));

    resolver::print_ips(&remote_ips);

    let mut iter = configs.into_iter().zip(remote_ips);
    let mut runners = vec![];

    while let Some((config, remote_ip)) = iter.next() {
        runners.push(tokio::spawn(run(config, remote_ip)));
    }

    join_all(runners).await;
}

pub async fn run(config: RelayConfig, remote_ip: Arc<RwLock<IpAddr>>) {
    let client_socket: SocketAddr =
        format!("{}:{}", config.listening_address, config.listening_port)
            .parse()
            .unwrap();
    let tcp_listener = net::TcpListener::bind(&client_socket).await.unwrap();

    let mut remote_socket: SocketAddr =
        format!("{}:{}", remote_ip.read().unwrap(), config.remote_port)
            .parse()
            .unwrap();

    // Start UDP connection
    let udp_remote_ip = remote_ip.clone();
    tokio::spawn(udp::transfer_udp(
        client_socket.clone(),
        remote_socket.port(),
        udp_remote_ip,
    ));

    // Start TCP connection
    loop {
        match tcp_listener.accept().await {
            Ok((inbound, _)) => {
                remote_socket = format!("{}:{}", &(remote_ip.read().unwrap()), config.remote_port)
                    .parse()
                    .unwrap();
                let transfer = transfer_tcp(inbound, remote_socket.clone()).map(|r| {
                    if let Err(_) = r {
                        return;
                    }
                });
                tokio::spawn(transfer);
            }
            Err(e) => {
                println!(
                    "TCP forward error {}:{}, {}",
                    config.remote_address, config.remote_port, e
                );
                break;
            }
        }
    }
}

async fn transfer_tcp(
    mut inbound: net::TcpStream,
    remote_socket: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut outbound = net::TcpStream::connect(remote_socket).await?;
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        io::copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        io::copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };

    try_join(client_to_server, server_to_client).await?;

    Ok(())
}
