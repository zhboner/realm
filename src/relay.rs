use futures::future::join_all;
use futures::future::try_join;
use futures::FutureExt;
use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::thread;
use tokio;
use tokio::io;
use tokio::net;

use crate::resolver;
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

    thread::spawn(move || resolver::resolve(remote_addrs, cloned_remote_ips));

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
    let mut tcp_listener = net::TcpListener::bind(&client_socket).await.unwrap();

    let mut remote_socket: SocketAddr =
        format!("{}:{}", remote_ip.read().unwrap(), config.remote_port)
            .parse()
            .unwrap();

    // Start UDP connection
    let udp_remote_ip = remote_ip.clone();
    thread::spawn(move || udp_transfer(client_socket.clone(), remote_socket.port(), udp_remote_ip));

    // Start TCP connection
    while let Ok((inbound, _)) = tcp_listener.accept().await {
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
}

// Two thread here
// 1. Receive packets and justify the forward destination. Then send packets to the second thread
// 2. Send all packets instructed by the first thread
fn udp_transfer(
    local_socket: SocketAddr,
    remote_port: u16,
    remote_ip: Arc<RwLock<IpAddr>>,
) -> Result<(), io::Error> {
    let sender = std::net::UdpSocket::bind(&local_socket).unwrap();
    let receiver = sender.try_clone().unwrap();
    let mut sender_vec = Vec::new();
    let (packet_sender, packet_receiver) = mpsc::channel::<([u8; 2048], usize, SocketAddr)>();

    // Start a new thread to send out packets
    thread::spawn(move || loop {
        if let Ok((data, size, client)) = packet_receiver.recv() {
            if let Err(e) = sender.send_to(&data[..size], client) {
                println!("failed to send UDP packet to {}, {}", client, e);
            }
        }
    });

    // Receive packets
    // Storing source ip in a FIFO queue to justify the forward destination
    // Send instruction to the above thread
    loop {
        let mut buf = [0u8; 2048];
        let (size, from) = receiver.recv_from(&mut buf).unwrap();

        let remote_socket: SocketAddr = format!("{}:{}", remote_ip.read().unwrap(), remote_port)
            .parse()
            .unwrap();

        match from != remote_socket {
            true => {
                // forward
                sender_vec.push(from);
                packet_sender
                    .send((buf, size, remote_socket.clone()))
                    .unwrap();
            }
            false => {
                // backward
                if sender_vec.len() < 1 {
                    continue;
                }
                let client_socket = sender_vec.remove(0);
                packet_sender.send((buf, size, client_socket)).unwrap();
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

    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);

    try_join(client_to_server, server_to_client).await?;

    Ok(())
}
