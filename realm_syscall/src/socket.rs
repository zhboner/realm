use std::io::Result;
use socket2::{Socket, Domain, Type};

/// Create a new socket. 
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
pub fn new_socket(domain: Domain, ty: Type) -> Result<Socket> {
    use std::os::unix::prelude::FromRawFd;
    use libc::{SOCK_NONBLOCK, SOCK_CLOEXEC};

    let fd = unsafe {
        libc::socket(
            domain.into(),
            libc::c_int::from(ty) | SOCK_NONBLOCK | SOCK_CLOEXEC,
            0,
        )
    };

    if fd < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { Socket::from_raw_fd(fd) })
    }
}

/// Create a new socket. 
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
pub fn new_socket(domain: Domain, ty: Type) -> Result<Socket> {
    let socket = Socket::new(domain, ty, None)?;
    socket.set_nonblocking(true)?;
    Ok(socket)
}

/// Bind a socket to a specific network interface.
/// 
/// It seems `SO_BINDTODEVICE` is not supported on BSDs, we should use `IP_SENDIF` instead.
/// 
/// Reference:
/// - [shadowsocks-rust](https://docs.rs/shadowsocks/1.13.1/src/shadowsocks/net/sys/unix/linux/mod.rs.html#256-276).
/// - [freebsd](https://lists.freebsd.org/pipermail/freebsd-net/2012-April/032064.html).
#[cfg(target_os = "linux")]
pub fn bind_to_device<T: std::os::unix::io::AsRawFd>(
    socket: &T,
    iface: &str,
) -> std::io::Result<()> {
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
