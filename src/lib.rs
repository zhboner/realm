pub mod cmd;
pub mod conf;
pub mod consts;
pub mod api;
pub use realm_core as core;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENV_CONFIG: &str = "REALM_CONF";
pub const ENV_API_KEY: &str = "REALM_API_KEY";
