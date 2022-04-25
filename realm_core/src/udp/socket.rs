use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;

use realm_syscall::new_udp_socket;
use tokio::net::UdpSocket;

use crate::dns::resolve_addr;
use crate::time::timeoutfut;
use crate::endpoint::ConnectOpts;

pub fn bind(laddr: &SocketAddr) -> Result<UdpSocket> {
    let socket = new_udp_socket(laddr)?;

    // ignore error
    let _ = socket.set_reuse_address(true);

    socket.bind(&laddr.clone().into())?;

    UdpSocket::from_std(socket.into())
}
