use std::io::Result;
use futures::future::join_all;

mod tcp;
use tcp::TcpListener;
use crate::utils::Endpoint;

pub async fn run(eps: Vec<Endpoint>) {
    let mut workers = Vec::with_capacity(compute_workers(&eps));
    for ep in eps.into_iter() {
        #[cfg(feature = "udp")]
        if ep.opts.use_udp {
            workers.push(tokio::spawn(proxy_udp(ep.clone())))
        }
        workers.push(tokio::spawn(proxy_tcp(ep)));
    }
    join_all(workers).await;
}

async fn proxy_tcp(ep: Endpoint) -> Result<()> {
    let Endpoint {
        local,
        remote,
        opts,
        ..
    } = ep;
    let lis = TcpListener::bind(local)
        .await
        .unwrap_or_else(|_| panic!("unable to bind {}", &local));
    while let Ok((stream, _)) = lis.accept().await {
        tokio::spawn(tcp::proxy(stream, remote.clone(), opts));
    }
    Ok(())
}

#[cfg(feature = "udp")]
mod udp;

#[cfg(feature = "udp")]
async fn proxy_udp(ep: Endpoint) -> Result<()> {
    let Endpoint {
        local,
        remote,
        opts,
        ..
    } = ep;
    udp::proxy(local, remote.clone(), opts).await
}

fn compute_workers(workers: &[Endpoint]) -> usize {
    macro_rules! num {
        ($x: expr) => {
            if !$x {
                1
            } else {
                2
            }
        };
    }
    workers
        .iter()
        .fold(0, |total, x| total + num!(x.opts.use_udp))
}
