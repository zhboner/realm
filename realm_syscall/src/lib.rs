//! Realm's convenient syscall collections.

#[cfg(unix)]
mod daemon;
#[cfg(unix)]
pub use daemon::*;

#[cfg(all(unix, not(target_os = "android")))]
mod nofile;
#[cfg(all(unix, not(target_os = "android")))]
pub use nofile::*;

mod socket;
pub use socket::*;
pub use socket2;
