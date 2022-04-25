//! TCP relay entrance.

mod socket;
mod middle;
mod plain;

use std::io::Result;
use crate::trick::Ref;
use crate::endpoint::Endpoint;

/// Launch a tcp relay.
pub async fn run_tcp(endpoint: Ref<Endpoint>) -> Result<()> {
    let Endpoint {
        listen,
        remote,
        conn_opts,
    } = endpoint.as_ref();

    let remote = Ref::new(remote);
    let conn_opts = Ref::new(conn_opts);

    let lis = socket::bind(listen)
        .unwrap_or_else(|e| panic!("[tcp]failed to bind {}: {}", listen, e));

    loop {
        let (stream, addr) = match lis.accept().await {
            Ok(x) => x,
            Err(e) => {
                log::error!("[tcp]failed to accept: {}", e);
                continue;
            }
        };

        let link_info = format!("{} => {}", &addr, remote.as_ref());
        log::info!("[tcp]{}", &link_info);

        let _ = stream.set_nodelay(true);

        tokio::spawn(async move {
            match middle::connect_and_relay(stream, remote, conn_opts).await {
                Ok(..) => log::debug!("[tcp]{}, finish", link_info),
                Err(e) => log::error!("[tcp]{}, error: {}", link_info, e),
            }
        });
    }
}
