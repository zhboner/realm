pub mod cmd;
pub mod dns;
pub mod conf;
pub mod utils;
pub mod relay;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENV_CONFIG: &str = "REALM_CONF";
