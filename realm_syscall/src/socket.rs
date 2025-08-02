use std::io::Result;
use std::net::SocketAddr;
use socket2::{Socket, Domain, Type, Protocol};

/// Create a new non-blocking socket.
///
/// On unix-like platforms, [`SOCK_NONBLOCK`](libc::SOCK_NONBLOCK) and
/// [`SOCK_CLOEXEC`](libc::SOCK_CLOEXEC) are assigned
/// when creating a new socket, which saves a [`fcntl`](libc::fcntl) syscall.
///
/// On other platforms, a socket is created without extra flags
/// then set to `non_blocking`.
#[cfg(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd"
))]
#[inline]
pub fn new_socket(domain: Domain, ty: Type, pt: Protocol) -> Result<Socket> {
    use std::os::unix::prelude::FromRawFd;
    use libc::{SOCK_NONBLOCK, SOCK_CLOEXEC};

    let fd = unsafe {
        libc::socket(
            domain.into(),
            libc::c_int::from(ty) | SOCK_NONBLOCK | SOCK_CLOEXEC,
            pt.into(),
        )
    };

    if fd < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { Socket::from_raw_fd(fd) })
    }
}

/// Create a new non-blocking socket.
#[cfg(not(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
#[inline]
pub fn new_socket(domain: Domain, ty: Type, pt: Protocol) -> Result<Socket> {
    let socket = Socket::new(domain, ty, pt)?;
    socket.set_nonblocking(true)?;
    Ok(socket)
}

/// Create a new non-blocking TCP socket.
///
/// On unix-like platforms, [`SOCK_NONBLOCK`](libc::SOCK_NONBLOCK) and
/// [`SOCK_CLOEXEC`](libc::SOCK_CLOEXEC) are assigned
/// when creating a new socket, which saves a [`fcntl`](libc::fcntl) syscall.
///
/// On other platforms, a socket is created without extra flags
/// then set to `non_blocking`.
#[inline]
pub fn new_tcp_socket(addr: &SocketAddr) -> Result<Socket> {
    let domain = match addr {
        SocketAddr::V4(..) => Domain::IPV4,
        SocketAddr::V6(..) => Domain::IPV6,
    };
    new_socket(domain, Type::STREAM, Protocol::TCP)
}

/// Create a new non-blocking MPTCP socket.
#[cfg(target_os = "linux")]
#[inline]
pub fn new_mptcp_socket(addr: &SocketAddr) -> Result<Socket> {
    let domain = match addr {
        SocketAddr::V4(..) => Domain::IPV4,
        SocketAddr::V6(..) => Domain::IPV6,
    };
    new_socket(domain, Type::STREAM, Protocol::MPTCP)
}

/// Create a new non-blocking UDP socket.
///
/// On unix-like platforms, [`SOCK_NONBLOCK`](libc::SOCK_NONBLOCK) and
/// [`SOCK_CLOEXEC`](libc::SOCK_CLOEXEC) are assigned
/// when creating a new socket, which saves a [`fcntl`](libc::fcntl) syscall.
///
/// On other platforms, a socket is created without extra flags
/// then set to `non_blocking`.
#[inline]
pub fn new_udp_socket(addr: &SocketAddr) -> Result<Socket> {
    let domain = match addr {
        SocketAddr::V4(..) => Domain::IPV4,
        SocketAddr::V6(..) => Domain::IPV6,
    };
    new_socket(domain, Type::DGRAM, Protocol::UDP)
}

/// Bind a socket to a specific network interface.
///
/// It seems `SO_BINDTODEVICE` is not supported on BSDs, we should use `IP_SENDIF` instead.
///
/// Reference:
/// - [shadowsocks-rust](https://docs.rs/shadowsocks/1.13.1/src/shadowsocks/net/sys/unix/linux/mod.rs.html#256-276).
/// - [freebsd](https://lists.freebsd.org/pipermail/freebsd-net/2012-April/032064.html).
#[cfg(target_os = "linux")]
pub fn bind_to_device<T: std::os::unix::io::AsRawFd>(socket: &T, iface: &str) -> std::io::Result<()> {
    let iface_bytes = iface.as_bytes();

    if unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_BINDTODEVICE,
            iface_bytes.as_ptr() as *const _ as *const libc::c_void,
            iface_bytes.len() as libc::socklen_t,
        )
    } < 0
    {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
