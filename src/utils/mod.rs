mod types;
pub use types::*;

mod consts;
pub use consts::*;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

#[allow(unused)]
#[inline]
pub fn new_sockaddr_v4() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
}

#[allow(unused)]
#[inline]
pub fn new_sockaddr_v6() -> SocketAddr {
    SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 0)
}

#[cfg(unix)]
pub fn daemonize() {
    use std::env::current_dir;
    use daemonize::Daemonize;

    let pwd = current_dir().unwrap().canonicalize().unwrap();
    let daemon = Daemonize::new()
        .umask(0)
        .working_directory(pwd)
        .exit_action(|| println!("realm is running in the background"));

    daemon
        .start()
        .unwrap_or_else(|e| eprintln!("failed to daemonize, {}", e));
}
