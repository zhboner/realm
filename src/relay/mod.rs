use std::net::SocketAddr;
use futures::future::join_all;

use tokio::io;
use tokio::net::TcpListener;

mod dns;
mod tcp;
mod udp;
mod types;

pub use types::{Endpoint, RemoteAddr};
pub use dns::init_resolver;

#[cfg(target_os = "linux")]
mod zero_copy;

pub async fn run(eps: Vec<Endpoint>) {
    let mut workers = vec![];
    for ep in eps.into_iter() {
        if ep.udp {
            workers.push(tokio::spawn(proxy_udp(ep.local, ep.remote.clone())))
        }
        workers.push(tokio::spawn(proxy_tcp(ep.local, ep.remote)));
    }
    join_all(workers).await;
}

async fn proxy_tcp(local: SocketAddr, remote: RemoteAddr) -> io::Result<()> {
    let lis = TcpListener::bind(&local).await.expect("unable to bind");
    while let Ok((stream, _)) = lis.accept().await {
        tokio::spawn(tcp::proxy(stream, remote.clone()));
    }
    Ok(())
}

async fn proxy_udp(local: SocketAddr, remote: RemoteAddr) -> io::Result<()> {
    udp::proxy(local, remote).await
}
