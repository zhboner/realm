use std::io::Result;
use std::net::SocketAddr;
use futures::future::join_all;

mod tcp;
use tcp::TcpListener;
use crate::utils::{Endpoint, Remote};

pub async fn run(eps: Vec<Endpoint>) {
    let mut workers = vec![];
    for ep in eps.into_iter() {
        #[cfg(feature = "udp")]
        if ep.udp {
            workers.push(tokio::spawn(proxy_udp(ep.local, ep.remote.clone())))
        }
        workers.push(tokio::spawn(proxy_tcp(ep.local, ep.remote)));
    }
    join_all(workers).await;
}

async fn proxy_tcp(local: SocketAddr, remote: Remote) -> Result<()> {
    let lis = TcpListener::bind(local).await.expect("unable to bind");
    while let Ok((stream, _)) = lis.accept().await {
        tokio::spawn(tcp::proxy(stream, remote.addr.clone(), remote.through));
    }
    Ok(())
}

#[cfg(feature = "udp")]
mod udp;

#[cfg(feature = "udp")]
async fn proxy_udp(local: SocketAddr, remote: Remote) -> Result<()> {
    udp::proxy(local, remote.addr.clone(), remote.through).await
}
