use std::io::Result;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use super::SockMap;
use super::BUF_SIZE;
use super::socket;

use crate::trick::Ref;
use crate::time::timeoutfut;
use crate::dns::resolve_addr;
use crate::endpoint::{RemoteAddr, ConnectOpts};

pub async fn associate_and_relay(
    lis: &UdpSocket,
    raddr: &RemoteAddr,
    conn_opts: &ConnectOpts,
    sockmap: &SockMap,
) -> Result<()> {
    let mut buf = vec![0u8; BUF_SIZE];
    let associate_timeout = conn_opts.associate_timeout;

    loop {
        let (n, laddr) = lis.recv_from(&mut buf).await?;
        log::debug!("[udp]recvfrom client {}", &laddr);

        let addr = resolve_addr(raddr).await?.iter().next().unwrap();
        log::debug!("[udp]{} resolved as {}", raddr, &addr);

        // get the socket associated with a unique client
        let remote = match sockmap.find(&laddr) {
            Some(x) => x,
            None => {
                log::info!(
                    "[udp]new association {} => {} as {}",
                    &laddr,
                    raddr,
                    &addr
                );

                let remote =
                    Arc::new(socket::associate(&addr, conn_opts).await?);

                sockmap.insert(laddr, remote.clone());

                // spawn sending back task
                tokio::spawn(send_back(
                    Ref::new(lis),
                    laddr,
                    remote.clone(),
                    Ref::new(sockmap),
                    associate_timeout,
                ));

                remote
            }
        };

        remote.send_to(&buf[..n], &addr).await?;
    }
}

async fn send_back(
    lis: Ref<UdpSocket>,
    laddr: SocketAddr,
    remote: Arc<UdpSocket>,
    sockmap: Ref<SockMap>,
    associate_timeout: usize,
) {
    let mut buf = vec![0u8; BUF_SIZE];

    loop {
        let res =
            match timeoutfut(remote.recv_from(&mut buf), associate_timeout)
                .await
            {
                Ok(x) => x,
                Err(_) => {
                    log::debug!("[udp]association for {} timeout", &laddr);
                    break;
                }
            };

        let (n, raddr) = match res {
            Ok(x) => x,
            Err(e) => {
                log::error!("[udp]failed to recvfrom remote: {}", e);
                continue;
            }
        };

        log::debug!("[udp]recvfrom remote {}", &raddr);

        if let Err(e) = lis.send_to(&buf[..n], &laddr).await {
            log::error!("[udp]failed to sendto client{}: {}", &laddr, e);
            continue;
        }
    }

    sockmap.remove(&laddr);
    log::debug!("[udp]remove association for {}", &laddr);
}
