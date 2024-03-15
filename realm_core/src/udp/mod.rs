//! UDP relay entrance.

mod socket;
mod sockmap;
mod middle;

use std::io::Result;

use crate::endpoint::Endpoint;

use sockmap::SockMap;
use middle::associate_and_relay;

/// UDP Buffer size.
pub const BUF_SIZE: usize = 2048;

/// Launch a udp relay.
pub async fn run_udp(endpoint: Endpoint) -> Result<()> {
    let Endpoint {
        laddr,
        raddr,
        bind_opts,
        conn_opts,
        ..
    } = endpoint;

    let sockmap = SockMap::new();

    let lis = socket::bind(&laddr, bind_opts).unwrap_or_else(|e| panic!("[udp]failed to bind {}: {}", laddr, e));

    loop {
        if let Err(e) = associate_and_relay(&lis, &raddr, &conn_opts, &sockmap).await {
            log::error!("[udp]error: {}", e);
        }
    }
}
