use std::io::Result;

use tokio::net::TcpStream;

use super::socket;
use super::plain;
use crate::trick::Ref;
use crate::endpoint::{RemoteAddr, ConnectOpts};

#[allow(unused_variables)]
pub async fn connect_and_relay(
    local: TcpStream,
    remote: Ref<RemoteAddr>,
    conn_opts: Ref<ConnectOpts>,
) -> Result<()> {
    // before connect
    // ..

    // connect!
    let remote = socket::connect(remote.as_ref(), conn_opts.as_ref()).await?;

    // after connected
    let res = plain::run_relay(local, remote).await;

    if let Err(e) = res {
        log::debug!("[tcp]forward error: {}, ignored", e);
    }

    Ok(())
}
