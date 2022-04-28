pub mod cmd;
pub mod conf;
pub mod relay;
pub mod consts;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENV_CONFIG: &str = "REALM_CONF";
