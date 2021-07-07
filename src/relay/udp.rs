use std::time::Duration;
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;

use tokio::io;
use tokio::net::UdpSocket;
use tokio::sync::oneshot;
use tokio::time::sleep;

use super::utils::RemoteAddr;

const BUFFERSIZE: usize = 2048;
const TIMEOUT: Duration = Duration::from_secs(60 * 15);

pub async fn proxy(local: SocketAddr, remote: RemoteAddr) -> io::Result<()> {
    // records (client_addr, alloc_socket)
    let mut record = HashMap::new();

    let local_socket = Arc::new(UdpSocket::bind(&local).await.unwrap());
    let mut buf = vec![0u8; BUFFERSIZE];
    loop {
        tokio::select! {
            _ = async {
                let (n, client_addr) = local_socket.recv_from(&mut buf).await?;
                if !record.contains_key(&client_addr) {
                    // pick a random port
                    let alloc_socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await.unwrap());
                    let (emit, cancel) = oneshot::channel::<()>();
                    tokio::spawn(send_back(
                        client_addr, local_socket.clone(), alloc_socket.clone(), cancel
                    ));
                    record.insert(client_addr, (alloc_socket,emit));
                }
                let (alloc_socket, _) = record.get(&client_addr).unwrap();
                let remote_addr = remote.to_sockaddr_as_ref().await?;
                alloc_socket.send_to(&buf[..n], &remote_addr).await?;
                Ok::<_, io::Error>(())
            } => {}
            _ = async { sleep(TIMEOUT).await } => record.clear()
        }
    }
}

async fn send_back(
    client_addr: SocketAddr,
    local_socket: Arc<UdpSocket>,
    alloc_socket: Arc<UdpSocket>,
    cancel: oneshot::Receiver<()>,
) -> io::Result<()> {
    let mut buf = vec![0u8; BUFFERSIZE];
    tokio::select! {
        ret = async {
            loop {
                let (n, _) = alloc_socket.recv_from(&mut buf).await?;
                local_socket.send_to(&buf[..n], &client_addr).await?;
            }
        } => { ret }
       _ = cancel => Ok(())
    }
}
