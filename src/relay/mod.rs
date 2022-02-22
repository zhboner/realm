use log::{debug, info, warn, error};
use futures::future::join_all;

mod tcp;
use tcp::TcpListener;

use crate::utils::Endpoint;
use crate::utils::{EndpointRef, RemoteAddrRef, ConnectOptsRef};

pub async fn run(endpoints: Vec<Endpoint>) {
    let mut workers = Vec::with_capacity(compute_workers(&endpoints));
    for endpoint in endpoints.iter() {
        #[cfg(feature = "udp")]
        if endpoint.opts.use_udp {
            workers.push(tokio::spawn(run_udp(endpoint.into())))
        }
        workers.push(tokio::spawn(run_tcp(endpoint.into())));
    }
    join_all(workers).await;
}

pub async fn run_tcp(endpoint: EndpointRef) {
    let Endpoint {
        listen,
        remote,
        opts,
        ..
    } = endpoint.as_ref();

    let remote: RemoteAddrRef = remote.into();
    let opts: ConnectOptsRef = opts.into();

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
            match tcp::connect_and_relay(stream, remote, opts).await {
                Ok(..) => debug!("[tcp]{}, finish", msg),
                Err(e) => error!("[tcp]{}, error: {}", msg, e),
            }
        });
    }
}

#[cfg(feature = "udp")]
mod udp;

#[cfg(feature = "udp")]
pub async fn run_udp(endpoint: EndpointRef) {
    let Endpoint {
        listen,
        remote,
        opts,
        ..
    } = endpoint.as_ref();

    if let Err(e) = udp::associate_and_relay(listen, remote, opts.into()).await
    {
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
