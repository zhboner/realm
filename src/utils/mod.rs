mod types;
pub use types::*;

mod consts;
pub use consts::*;

mod timeout;
pub use timeout::*;

mod ex_types;
pub use ex_types::*;

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
        .unwrap_or_else(|e| eprintln!("failed to daemonize: {}", e));
}

// refer to
// https://man7.org/linux/man-pages/man2/setrlimit.2.html
// https://github.com/shadowsocks/shadowsocks-rust/blob/master/crates/shadowsocks-service/src/sys/unix/mod.rs
#[cfg(all(unix, not(target_os = "android")))]
pub fn set_nofile_limit(nofile: u64) {
    use libc::RLIMIT_NOFILE;
    use libc::{rlimit, rlim_t};
    use std::io::Error;

    let lim = rlimit {
        rlim_cur: nofile as rlim_t,
        rlim_max: nofile as rlim_t,
    };

    if unsafe { libc::setrlimit(RLIMIT_NOFILE, &lim as *const _) } < 0 {
        eprintln!("failed to set nofile limit: {}", Error::last_os_error());
    } else {
        println!("set nofile limit to {}", nofile);
    }
}

// refer to
// https://man7.org/linux/man-pages/man2/setrlimit.2.html
#[cfg(all(unix, not(target_os = "android")))]
pub fn get_nofile_limit() -> Option<(u64, u64)> {
    use libc::RLIMIT_NOFILE;
    use libc::rlimit;
    use std::io::Error;

    let mut lim = rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    if unsafe { libc::getrlimit(RLIMIT_NOFILE, &mut lim as *mut _) } < 0 {
        eprintln!("failed to get nofile limit: {}", Error::last_os_error());
        return None;
    };

    Some((lim.rlim_cur as u64, lim.rlim_max as u64))
}
