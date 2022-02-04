use log::{info, error};
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

async fn proxy_tcp(ep: Endpoint) {
    let Endpoint {
        listen,
        remote,
        opts,
        ..
    } = ep;

    let lis = TcpListener::bind(listen)
        .await
        .unwrap_or_else(|e| panic!("unable to bind {}: {}", &listen, &e));

    loop {
        let (stream, addr) = match lis.accept().await {
            Ok(x) => x,
            Err(e) => {
                error!("[tcp]failed to accept: {}", &e);
                continue;
            }
        };

        let msg = format!("{} => {}", &addr, &remote);
        info!("[tcp]{}", &msg);

        let remote = remote.clone();
        tokio::spawn(async move {
            match tcp::proxy(stream, remote, opts).await {
                Ok((up, dl)) => info!(
                    "[tcp]{} finish, upload: {}b, download: {}b",
                    msg, up, dl
                ),
                Err(e) => error!("[tcp]{} error: {}", msg, e),
            }
        });
    }
}

#[cfg(feature = "udp")]
mod udp;

#[cfg(feature = "udp")]
async fn proxy_udp(ep: Endpoint) {
    let Endpoint {
        listen,
        remote,
        opts,
        ..
    } = ep;

    if let Err(e) = udp::proxy(listen, remote, opts).await {
        panic!("udp forward exit: {}", &e);
    }
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
