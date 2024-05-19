//! UDP relay entrance.

mod socket;
mod sockmap;
mod middle;
mod batched;

use std::io::Result;

use crate::trick::Ref;
use crate::endpoint::Endpoint;

use sockmap::SockMap;
use middle::associate_and_relay;

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

    let lis = Ref::new(&lis);
    let raddr = Ref::new(&raddr);
    let conn_opts = Ref::new(&conn_opts);
    let sockmap = Ref::new(&sockmap);
    loop {
        if let Err(e) = associate_and_relay(lis, raddr, conn_opts, sockmap).await {
            log::error!("[udp]error: {}", e);
        }
    }
}
