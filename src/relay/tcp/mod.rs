mod zio;
use cfg_if::cfg_if;

#[cfg(feature = "proxy-protocol")]
mod haproxy;

cfg_if! {
    if #[cfg(feature = "tfo")] {
        mod tfo;
        use tfo::TcpStream;
        pub use tfo::TcpListener;
    } else {
        use tokio::net::TcpStream;
        pub use tokio::net::TcpListener;
    }
}

use std::io::Result;

use log::debug;

use tokio::net::TcpSocket;

use crate::utils::socket;
use crate::utils::ConnectOpts;
use crate::utils::{RemoteAddrRef, ConnectOptsRef};

#[allow(unused_variables)]
pub async fn connect_and_relay(
    mut inbound: TcpStream,
    remote: RemoteAddrRef,
    conn_opts: ConnectOptsRef,
) -> Result<()> {
    let ConnectOpts {
        fast_open,
        zero_copy,
        send_through,
        bind_interface,
        haproxy_opts,
        ..
    } = conn_opts.as_ref();

    // before connect
    let remote = remote.to_sockaddr().await?;
    debug!("[tcp]remote resolved as {}", &remote);

    let socket = socket::new_socket(socket::Type::STREAM, &remote, &conn_opts)?;
    let socket = TcpSocket::from_std_stream(socket.into());

    // connect!
    #[cfg(not(feature = "tfo"))]
    let mut outbound = socket.connect(remote).await?;

    #[cfg(feature = "tfo")]
    let mut outbound = if *fast_open {
        TcpStream::connect_with_socket(socket, remote).await?
    } else {
        socket.connect(remote).await?.into()
    };

    // after connected
    #[cfg(feature = "proxy-protocol")]
    if haproxy_opts.send_proxy || haproxy_opts.accept_proxy {
        haproxy::handle_proxy_protocol(
            &mut inbound,
            &mut outbound,
            *haproxy_opts,
        )
        .await?;
    }

    #[cfg(all(target_os = "linux", feature = "zero-copy"))]
    let res = if *zero_copy {
        zio::bidi_copy_pipe(&mut inbound, &mut outbound).await
    } else {
        zio::bidi_copy_buffer(&mut inbound, &mut outbound).await
    };

    #[cfg(not(all(target_os = "linux", feature = "zero-copy")))]
    let res = zio::bidi_copy_buffer(&mut inbound, &mut outbound).await;

    if let Err(e) = res {
        debug!("[tcp]forward error: {}, ignored", e);
    }

    Ok(())
}
