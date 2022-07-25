//! Realm's core facilities.

pub mod dns;
pub mod tcp;
pub mod udp;
pub mod time;
pub mod trick;
pub mod endpoint;

#[cfg(feature = "hook")]
pub use realm_hook as hook;

#[cfg(feature = "balance")]
pub use realm_lb as balance;

#[cfg(feature = "transport")]
pub use kaminari;
