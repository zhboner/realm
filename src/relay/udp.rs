use std::io::Result;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use log::{debug, info, error};

use tokio::net::UdpSocket;

use crate::utils::DEFAULT_BUF_SIZE;

use crate::utils::{Ref, RemoteAddr, ConnectOpts};

use crate::utils::timeoutfut;
use crate::utils::socket;

// client <--> allocated socket

type SockMap = RwLock<HashMap<SocketAddr, Arc<UdpSocket>>>;

const BUF_SIZE: usize = DEFAULT_BUF_SIZE;

pub fn new_sock_map() -> SockMap {
    RwLock::new(HashMap::new())
}

pub async fn associate_and_relay(
    sock_map: &SockMap,
    listen_sock: &UdpSocket,
    remote_addr: &RemoteAddr,
    conn_opts: Ref<ConnectOpts>,
) -> Result<()> {
    let timeout = conn_opts.udp_timeout;
    let mut buf = vec![0u8; BUF_SIZE];

    loop {
        let (n, client_addr) = listen_sock.recv_from(&mut buf).await?;

        debug!("[udp]recvfrom client {}", &client_addr);

        let remote_addr = remote_addr.to_sockaddr().await?;

        // get the socket associated with a unique client
        let alloc_sock = match find_socket(sock_map, &client_addr) {
            Some(x) => x,
            None => {
                info!("[udp]{} => {}", &client_addr, &remote_addr);

                let socket = socket::new_socket(socket::Type::DGRAM, &remote_addr, &conn_opts)?;

                // from_std panics only when tokio runtime not setup
                let new_sock = Arc::new(UdpSocket::from_std(socket.into()).unwrap());

                tokio::spawn(send_back(
                    sock_map.into(),
                    client_addr,
                    listen_sock.into(),
                    new_sock.clone(),
                    timeout,
                ));

                insert_socket(sock_map, client_addr, new_sock.clone());
                new_sock
            }
        };

        alloc_sock.send_to(&buf[..n], &remote_addr).await?;
    }
}

async fn send_back(
    sock_map: Ref<SockMap>,
    client_addr: SocketAddr,
    listen_sock: Ref<UdpSocket>,
    alloc_sock: Arc<UdpSocket>,
    timeout: usize,
) {
    let mut buf = vec![0u8; BUF_SIZE];

    loop {
        let res = match timeoutfut(alloc_sock.recv_from(&mut buf), timeout).await {
            Ok(x) => x,
            Err(_) => {
                debug!("[udp]association for {} timeout", &client_addr);
                break;
            }
        };

        let (n, remote_addr) = match res {
            Ok(x) => x,
            Err(e) => {
                error!("[udp]failed to recvfrom remote: {}", e);
                continue;
            }
        };

        debug!("[udp]recvfrom remote {}", &remote_addr);

        if let Err(e) = listen_sock.send_to(&buf[..n], &client_addr).await {
            error!("[udp]failed to sendto client{}: {}", &client_addr, e);
            continue;
        }
    }

    sock_map.write().unwrap().remove(&client_addr);
    debug!("[udp]remove association for {}", &client_addr);
}

#[inline]
fn find_socket(sock_map: &SockMap, client_addr: &SocketAddr) -> Option<Arc<UdpSocket>> {
    // fetch the lock

    let alloc_sock = sock_map.read().unwrap();

    alloc_sock.get(client_addr).cloned()

    // drop the lock
}

#[inline]
fn insert_socket(sock_map: &SockMap, client_addr: SocketAddr, new_sock: Arc<UdpSocket>) {
    // fetch the lock

    sock_map.write().unwrap().insert(client_addr, new_sock);

    // drop the lock
}
