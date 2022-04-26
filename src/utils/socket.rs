mod detail {

    use std::io::Result;
    use socket2::{Socket, Domain, Type};

    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn new_socket(domain: Domain, ty: Type) -> Result<Socket> {
        use std::os::unix::prelude::FromRawFd;

        use libc::{SOCK_NONBLOCK, SOCK_CLOEXEC};

        let ty = libc::c_int::from(ty) | SOCK_NONBLOCK | SOCK_CLOEXEC;

        let fd = unsafe { libc::socket(domain.into(), ty | SOCK_NONBLOCK | SOCK_CLOEXEC, 0) };

        if fd < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(unsafe { Socket::from_raw_fd(fd) })
        }
    }

    #[cfg(not(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    pub fn new_socket(domain: Domain, ty: Type) -> Result<Socket> {
        let socket = Socket::new(domain, ty, None)?;
        socket.set_nonblocking(true)?;
        Ok(socket)
    }
}

use std::io::Result;
use std::net::SocketAddr;

use log::warn;
use socket2::{Socket, SockAddr};
use super::ConnectOpts;

pub use socket2::{Type, Domain};

pub fn new_socket(ty: Type, addr: &SocketAddr, opts: &ConnectOpts) -> Result<Socket> {
    let domain = match addr {
        SocketAddr::V4(..) => Domain::IPV4,
        SocketAddr::V6(..) => Domain::IPV6,
    };

    macro_rules! try_opt {
        ($op: expr, $msg: expr) => {{
            if let Err(e) = $op {
                warn!("[sys]$msg: {}", e);
            }
        }};
    }

    let socket = detail::new_socket(domain, ty)?;

    try_opt!(
        socket.set_reuse_address(true),
        "failed to set reuse_addr option for new socket"
    );

    if ty == Type::STREAM {
        try_opt!(socket.set_nodelay(true), "failed to set no_delay option for new socket");
    }

    #[cfg(target_os = "linux")]
    if let Some(iface) = &opts.bind_interface {
        try_opt!(
            crate::utils::bind_to_device(&socket, iface),
            "failed to bind new socket to device"
        );
    }

    if let Some(addr) = &opts.send_through {
        try_opt!(
            socket.bind(&SockAddr::from(*addr)),
            "failed to bind new socket to address"
        )
    }

    Ok(socket)
}
