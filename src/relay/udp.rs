use std::io::Result;
use std::time::Duration;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::utils::{RemoteAddr, ConnectOpts};
use crate::utils::{new_sockaddr_v4, new_sockaddr_v6};

// client <--> allocated socket
type SockMap = Arc<RwLock<HashMap<SocketAddr, Arc<UdpSocket>>>>;
const BUFFERSIZE: usize = 0x1000;
const TIMEOUT: Duration = Duration::from_secs(20);

pub async fn proxy(
    local: SocketAddr,
    remote: RemoteAddr,
    conn_opts: ConnectOpts,
) -> Result<()> {
    let ConnectOpts { send_through, .. } = conn_opts;
    let sock_map: SockMap = Arc::new(RwLock::new(HashMap::new()));
    let local_sock = Arc::new(UdpSocket::bind(&local).await.unwrap());
    let mut buf = vec![0u8; BUFFERSIZE];

    loop {
        let (n, client_addr) = local_sock.recv_from(&mut buf).await?;

        let remote_addr = remote.to_sockaddr().await?;

        // the socket associated with a unique client
        let alloc_sock = match get_socket(&sock_map, &client_addr) {
            Some(x) => x,
            None => {
                alloc_new_socket(
                    &sock_map,
                    client_addr,
                    &remote_addr,
                    &send_through,
                    local_sock.clone(),
                )
                .await
            }
        };

        alloc_sock.send_to(&buf[..n], &remote_addr).await?;
    }
}

async fn send_back(
    sock_map: SockMap,
    client_addr: SocketAddr,
    local_sock: Arc<UdpSocket>,
    alloc_sock: Arc<UdpSocket>,
) {
    let mut buf = vec![0u8; BUFFERSIZE];

    while let Ok(Ok((n, _))) =
        timeout(TIMEOUT, alloc_sock.recv_from(&mut buf)).await
    {
        if local_sock.send_to(&buf[..n], &client_addr).await.is_err() {
            break;
        }
    }

    sock_map.write().unwrap().remove(&client_addr);
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
    local_sock: Arc<UdpSocket>,
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
        local_sock,
        alloc_sock.clone(),
    ));

    sock_map
        .write()
        .unwrap()
        .insert(client_addr, alloc_sock.clone());
    alloc_sock
    // drop the lock
}
