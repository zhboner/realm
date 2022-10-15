pub mod cmd;
pub mod conf;
pub mod consts;
pub use realm_core as core;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENV_CONFIG: &str = "REALM_CONF";
