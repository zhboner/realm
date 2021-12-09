use std::io::Result;
use std::time::Duration;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use log::{debug, info, error};

use tokio::net::UdpSocket;
use tokio::time::timeout as timeoutfut;

use crate::utils::DEFAULT_BUF_SIZE;
use crate::utils::{RemoteAddr, ConnectOpts};
use crate::utils::{new_sockaddr_v4, new_sockaddr_v6};

// client <--> allocated socket
type SockMap = Arc<RwLock<HashMap<SocketAddr, Arc<UdpSocket>>>>;
const BUF_SIZE: usize = DEFAULT_BUF_SIZE;

pub async fn proxy(
    listen: SocketAddr,
    remote: RemoteAddr,
    conn_opts: ConnectOpts,
) -> Result<()> {
    let ConnectOpts {
        send_through,
        udp_timeout: timeout,
        ..
    } = conn_opts;
    let sock_map: SockMap = Arc::new(RwLock::new(HashMap::new()));
    let listen_sock = Arc::new(UdpSocket::bind(&listen).await?);
    let timeout = Duration::from_secs(timeout as u64);
    let mut buf = vec![0u8; BUF_SIZE];

    loop {
        let (n, client_addr) = match listen_sock.recv_from(&mut buf).await {
            Ok(x) => x,
            Err(e) => {
                error!("failed to recv udp packet from client: {}", &e);
                continue;
            }
        };

        debug!("recv udp packet from {}", &client_addr);

        let remote_addr = match remote.to_sockaddr().await {
            Ok(x) => x,
            Err(e) => {
                error!("failed to resolve remote addr: {}", &e);
                continue;
            }
        };

        // the old/new socket associated with a unique client
        let alloc_sock = match get_socket(&sock_map, &client_addr) {
            Some(x) => x,
            None => {
                info!("new udp association for client {}", &client_addr);
                alloc_new_socket(
                    &sock_map,
                    client_addr,
                    &remote_addr,
                    &send_through,
                    listen_sock.clone(),
                    timeout,
                )
                .await
            }
        };

        if let Err(e) = alloc_sock.send_to(&buf[..n], &remote_addr).await {
            error!("failed to send udp packet to remote: {}", &e);
        }
    }

    // Err(Error::new(ErrorKind::Other, "unknown error"))
}

async fn send_back(
    sock_map: SockMap,
    client_addr: SocketAddr,
    listen_sock: Arc<UdpSocket>,
    alloc_sock: Arc<UdpSocket>,
    timeout: Duration,
) {
    let mut buf = vec![0u8; BUF_SIZE];

    loop {
        let res =
            match timeoutfut(timeout, alloc_sock.recv_from(&mut buf)).await {
                Ok(x) => x,
                Err(_) => {
                    info!("udp association for {} timeout", &client_addr);
                    break;
                }
            };

        let (n, remote_addr) = match res {
            Ok(x) => x,
            Err(e) => {
                error!("failed to recv udp packet from remote: {}", &e);
                continue;
            }
        };

        debug!("recv udp packet from remote: {}", &remote_addr);

        if let Err(e) = listen_sock.send_to(&buf[..n], &client_addr).await {
            error!("failed to send udp packet back to client: {}", &e);
            continue;
        }
    }

    sock_map.write().unwrap().remove(&client_addr);
    info!("remove udp association for {}", &client_addr);
}

#[inline]
fn get_socket(
    sock_map: &SockMap,
    client_addr: &SocketAddr,
) -> Option<Arc<UdpSocket>> {
    let alloc_sock = sock_map.read().unwrap();
    alloc_sock.get(client_addr).cloned()
    // drop the lock
}

async fn alloc_new_socket(
    sock_map: &SockMap,
    client_addr: SocketAddr,
    remote_addr: &SocketAddr,
    send_through: &Option<SocketAddr>,
    listen_sock: Arc<UdpSocket>,
    timeout: Duration,
) -> Arc<UdpSocket> {
    // pick a random port
    let alloc_sock = Arc::new(match send_through {
        Some(x) => UdpSocket::bind(x).await.unwrap(),
        None => match remote_addr {
            SocketAddr::V4(_) => {
                UdpSocket::bind(new_sockaddr_v4()).await.unwrap()
            }
            SocketAddr::V6(_) => {
                UdpSocket::bind(new_sockaddr_v6()).await.unwrap()
            }
        },
    });
    // new send back task
    tokio::spawn(send_back(
        sock_map.clone(),
        client_addr,
        listen_sock,
        alloc_sock.clone(),
        timeout,
    ));

    sock_map
        .write()
        .unwrap()
        .insert(client_addr, alloc_sock.clone());
    alloc_sock
    // drop the lock
}
