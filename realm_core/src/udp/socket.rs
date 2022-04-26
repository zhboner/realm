use std::io::Result;
use std::net::SocketAddr;

use tokio::net::UdpSocket;
use realm_syscall::new_udp_socket;

use crate::endpoint::ConnectOpts;

pub fn bind(laddr: &SocketAddr) -> Result<UdpSocket> {
    let socket = new_udp_socket(laddr)?;

    // ignore error
    let _ = socket.set_reuse_address(true);

    socket.bind(&laddr.clone().into())?;

    UdpSocket::from_std(socket.into())
}

pub async fn associate(raddr: &SocketAddr, conn_opts: &ConnectOpts) -> Result<UdpSocket> {
    let ConnectOpts {
        bind_address,
        bind_interface,
        ..
    } = conn_opts;

    let socket = new_udp_socket(&raddr)?;

    // ignore error
    let _ = socket.set_reuse_address(true);

    if let Some(addr) = *bind_address {
        socket.bind(&addr.into())?;
    }

    #[cfg(target_os = "linux")]
    if let Some(iface) = bind_interface {
        realm_syscall::bind_to_device(&socket, iface)?;
    }

    UdpSocket::from_std(socket.into())
}
