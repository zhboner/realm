//! UDP relay entrance.

mod socket;
mod sockmap;
mod middle;

use crate::trick::Ref;
use crate::endpoint::Endpoint;

use sockmap::SockMap;
use middle::associate_and_relay;

pub const BUF_SIZE: usize = 2048;

pub async fn run_udp(endpoint: Ref<Endpoint>) {
    let Endpoint {
        listen,
        remote,
        conn_opts,
        ..
    } = endpoint.as_ref();

    let sockmap = SockMap::new();

    let lis = socket::bind(listen)
        .unwrap_or_else(|e| panic!("[udp]failed to bind {}: {}", listen, e));

    loop {
        if let Err(e) =
            associate_and_relay(&lis, remote, conn_opts, &sockmap).await
        {
            log::error!("[udp]error: {}", e);
        }
    }
}
