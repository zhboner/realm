use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;

use realm_syscall::new_tcp_socket;
use tokio::net::{TcpSocket, TcpStream, TcpListener};

use crate::dns::resolve_addr;
use crate::time::timeoutfut;
use crate::endpoint::{RemoteAddr, ConnectOpts};

#[allow(clippy::clone_on_copy)]
pub fn bind(laddr: &SocketAddr) -> Result<TcpListener> {
    let socket = new_tcp_socket(laddr)?;

    // ignore error
    let _ = socket.set_reuse_address(true);

    socket.bind(&laddr.clone().into())?;
    socket.listen(1024)?;

    TcpListener::from_std(socket.into())
}

pub async fn connect(raddr: &RemoteAddr, conn_opts: &ConnectOpts) -> Result<TcpStream> {
    let ConnectOpts {
        connect_timeout,
        bind_address,

        #[cfg(target_os = "linux")]
        bind_interface,
        ..
    } = conn_opts;

    let mut last_err = None;

    for addr in resolve_addr(raddr).await?.iter() {
        log::debug!("[tcp]{} resolved as {}", raddr, &addr);

        let socket = new_tcp_socket(&addr)?;

        // ignore error
        let _ = socket.set_nodelay(true);
        let _ = socket.set_reuse_address(true);

        if let Some(addr) = *bind_address {
            socket.bind(&addr.into())?;
        }

        #[cfg(target_os = "linux")]
        if let Some(iface) = bind_interface {
            realm_syscall::bind_to_device(&socket, iface)?;
        }

        let socket = TcpSocket::from_std_stream(socket.into());

        match timeoutfut(socket.connect(addr), *connect_timeout).await {
            Ok(Ok(stream)) => {
                log::debug!("[tcp]connect to {} as {}", raddr, &addr,);
                return Ok(stream);
            }
            Ok(Err(e)) => {
                log::warn!("[tcp]connect to {} as {}: {}, try next ip", raddr, &addr, &e);
                last_err = Some(e);
            }
            Err(_) => log::warn!("[tcp]connect to {} as {} timeout, try next ip", raddr, &addr),
        }
    }

    Err(last_err.unwrap_or_else(|| Error::new(ErrorKind::InvalidInput, "could not connect to any address")))
}
