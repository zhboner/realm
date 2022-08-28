use std::io::{Error, Result};
use libc::{rlimit, rlim_t, RLIMIT_NOFILE};

/// Set nofile limits.
///
/// `CAP_NET_ADMIN` privilege is required if exceeds hard limitation.
///
/// Reference:
/// - [man](https://man7.org/linux/man-pages/man2/setrlimit.2.html)
/// - [shadowsocks-rust](https://github.com/shadowsocks/shadowsocks-rust/blob/master/crates/shadowsocks-service/src/sys/unix/mod.rs)
#[cfg(all(unix, not(target_os = "android")))]
pub fn set_nofile_limit(nofile: u64) -> Result<()> {
    let lim = rlimit {
        rlim_cur: nofile as rlim_t,
        rlim_max: nofile as rlim_t,
    };

    if unsafe { libc::setrlimit(RLIMIT_NOFILE, &lim as *const _) } < 0 {
        Err(Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Get current nofile limits.
///
/// Reference: [man](https://man7.org/linux/man-pages/man2/setrlimit.2.html).
#[cfg(all(unix, not(target_os = "android")))]
pub fn get_nofile_limit() -> Result<(u64, u64)> {
    let mut lim = rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };

    if unsafe { libc::getrlimit(RLIMIT_NOFILE, &mut lim as *mut _) } < 0 {
        Err(Error::last_os_error())
    } else {
        Ok((lim.rlim_cur as u64, lim.rlim_max as u64))
    }
}

/// Bump nofile limits.
#[cfg(all(unix, not(target_os = "android")))]
pub fn bump_nofile_limit() -> Result<()> {
    let (cur, max) = get_nofile_limit()?;
    if cur < max {
        set_nofile_limit(max)?;
    }
    Ok(())
}
