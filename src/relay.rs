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

// Initialize DNS recolver
// Set up channel between listener and resolver
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
    let mut tcp_listener = net::TcpListener::bind(&client_socket).await?;

    // Receive the resolved remote ip
    // Detect changes of ip in the TCP loop
    // Then send changes to UDP thread
    let mut remote_ip = recv.recv().unwrap();
    let mut remote_socket: SocketAddr = format!("{}:{}", remote_ip, config.remote_port)
        .parse()
        .unwrap();

    // flow resolved address to UDP thread
    let (resolver_sender, resolver_receiver) = mpsc::channel::<SocketAddr>();

    // Start UDP connection
    thread::spawn(move || udp_transfer(client_socket.clone(), remote_socket.clone(), resolver_receiver));

    // Start TCP connection
    while let Ok((inbound, _)) = tcp_listener.accept().await {
        if let Ok(new_ip) = recv.try_recv() {
            remote_ip = new_ip;
            remote_socket = format!("{}:{}", &remote_ip, config.remote_port)
                .parse()
                .unwrap();
            resolver_sender.send(remote_socket.clone()).unwrap();
        }
        let transfer = transfer_tcp(inbound, remote_socket.clone()).map(|r| {
            if let Err(_) = r {
                return;
            }
        });
        tokio::spawn(transfer);
    }
    Ok(())
}

// Two thread here
// 1. Receive packets and justify the forward destination. Then send packets to the second thread
// 2. Send all packets instructed by the first thread
fn udp_transfer(local_socket: SocketAddr, remote_socket: SocketAddr, resolver_receiver: mpsc::Receiver<SocketAddr>) -> Result<(), io::Error> {
    let mut remote_socket = remote_socket;
    let sender = std::net::UdpSocket::bind(&local_socket).unwrap();
    let receiver = sender.try_clone().unwrap();
    let mut sender_vec = Vec::new();
    let (packet_sender, packet_receiver) = mpsc::channel::<([u8; 2048], usize, SocketAddr)>();

    // Start a new thread to send out packets
    thread::spawn(move || {
        loop {
            if let Ok((data, size, client)) = packet_receiver.recv() {
                if let Err(e) = sender.send_to(&data[..size], client) {
                    println!("failed to send out UDP packet, {}", e);
                }
            }
        }
    });

    // Receive packets 
    // Storing source ip in a FIFO queue to justify the forward destination
    // Send instruction to the above thread
    loop {
        let mut buf = [0u8; 2048];
        let (size, from) = receiver.recv_from(&mut buf).unwrap();
        if let Ok(new_remote_socket) = resolver_receiver.try_recv() {
            remote_socket = new_remote_socket;
        }
        match from != remote_socket {
            true => {           // forward
                sender_vec.push(from);
                packet_sender.send((buf, size, remote_socket.clone())).unwrap();
            }
            false => {          // backward
                if sender_vec.len() < 1 {continue}
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
