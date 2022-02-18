use log::{info, warn, error};
use futures::future::join_all;

mod tcp;
use tcp::TcpListener;

use crate::utils::Endpoint;
use crate::utils::{EndpointX, RemoteAddrX, ConnectOptsX};

pub async fn run(eps: Vec<Endpoint>) {
    let mut workers = Vec::with_capacity(compute_workers(&eps));
    for ep in eps.iter() {
        #[cfg(feature = "udp")]
        if ep.opts.use_udp {
            workers.push(tokio::spawn(proxy_udp(ep.into())))
        }
        workers.push(tokio::spawn(proxy_tcp(ep.into())));
    }
    join_all(workers).await;
}

async fn proxy_tcp(ep: EndpointX) {
    let Endpoint {
        listen,
        remote,
        opts,
        ..
    } = ep.as_ref();

    let remote: RemoteAddrX = remote.into();
    let opts: ConnectOptsX = opts.into();

    let lis = TcpListener::bind(*listen)
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

        let msg = format!("{} => {}", &addr, remote.as_ref());
        info!("[tcp]{}", &msg);

        if let Err(e) = stream.set_nodelay(true) {
            warn!(
                "[tcp]failed to set no_delay option for incoming stream: {}",
                e
            );
        }

        tokio::spawn(async move {
            match tcp::proxy(stream, remote, opts).await {
                Ok((up, dl)) => info!(
                    "[tcp]{} finish, upload: {}b, download: {}b",
                    msg, up, dl
                ),
                Err(e) => error!("[tcp]{}, error: {}", msg, e),
            }
        });
    }
}

#[cfg(feature = "udp")]
mod udp;

#[cfg(feature = "udp")]
async fn proxy_udp(ep: EndpointX) {
    let Endpoint {
        listen,
        remote,
        opts,
        ..
    } = ep.as_ref();

    if let Err(e) = udp::proxy(listen, remote, opts.into()).await {
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
