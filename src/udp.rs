use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio;
use tokio::io;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::time::sleep;

const BUFFERSIZE: usize = 2048;
const TIMEOUT: Duration = Duration::from_secs(60 * 15);

pub async fn transfer_udp(
    local_addr: SocketAddr,
    remote_port: u16,
    remote_ip: Arc<RwLock<IpAddr>>,
) -> io::Result<()> {
    // client_addr -> allocated_socket
    let mut record = HashMap::new();
    let local_socket = Arc::new(UdpSocket::bind(&local_addr).await.unwrap());
    let mut buf = vec![0u8; BUFFERSIZE];
    loop {
        tokio::select! {
            _ = async {
                let (n, client_addr) = local_socket.recv_from(&mut buf).await?;
                if !record.contains_key(&client_addr) {
                    // pick a random port
                    let allocated_socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await.unwrap());
                    let cloned_socket = allocated_socket.clone();
                    let (tx, rx) = oneshot::channel::<()>();
                    record.insert(client_addr, (allocated_socket,tx));
                    tokio::spawn(send_back(
                        client_addr, local_socket.clone(), cloned_socket, rx
                    ));
                }
                let (allocated_socket, _) = record.get(&client_addr).unwrap();
                let remote_addr: SocketAddr = format!("{}:{}", remote_ip.read().unwrap()
                , remote_port).parse().unwrap();
                allocated_socket.send_to(&buf[..n], &remote_addr).await?;
                Ok::<_, io::Error>(())
            } => {}
            _ = async { sleep(TIMEOUT).await } => record.clear()
        }
    }
}

async fn send_back(
    client_addr: SocketAddr,
    local_socket: Arc<UdpSocket>,
    allocated_socket: Arc<UdpSocket>,
    rx: oneshot::Receiver<()>,
) -> io::Result<()> {
    let mut buf = vec![0u8; BUFFERSIZE];
    tokio::select! {
        ret = async {
            loop {
                let (n, _) = allocated_socket.recv_from(&mut buf).await?;
                local_socket.send_to(&buf[..n], &client_addr).await?;
            }
        } => { ret }
       _ = rx => Ok(())
    }
}
