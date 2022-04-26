//! UDP relay entrance.

mod socket;
mod sockmap;
mod middle;

use crate::trick::Ref;
use crate::endpoint::Endpoint;

use sockmap::SockMap;
use middle::associate_and_relay;

/// UDP Buffer size.
pub const BUF_SIZE: usize = 2048;

/// Launch a udp relay.
pub async fn run_udp(endpoint: Ref<Endpoint>) {
    let Endpoint {
        laddr,
        raddr,
        conn_opts,
        ..
    } = endpoint.as_ref();

    let sockmap = SockMap::new();

    let lis = socket::bind(laddr).unwrap_or_else(|e| panic!("[udp]failed to bind {}: {}", laddr, e));

    loop {
        if let Err(e) = associate_and_relay(&lis, raddr, conn_opts, &sockmap).await {
            log::error!("[udp]error: {}", e);
        }
    }
}
