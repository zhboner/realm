use std::fmt::{Display, Formatter};

// default logfile
pub const DEFAULT_LOG_FILE: &str = "stdout";

// default timeout
pub const TCP_TIMEOUT: usize = 5;
pub const TCP_KEEPALIVE: usize = 15;
pub const TCP_KEEPALIVE_PROBE: usize = 3;
pub const UDP_TIMEOUT: usize = 30;

// default haproxy proxy-protocol version
pub const PROXY_PROTOCOL_VERSION: usize = 2;

// default haproxy proxy-protocol version
pub const PROXY_PROTOCOL_TIMEOUT: usize = 5;

// features
macro_rules! def_feat {
    ($fet: ident, $name: expr) => {
        pub const $fet: bool = if cfg!(feature = $name) { true } else { false };
    };
}

def_feat!(FEATURE_HOOK, "hook");
def_feat!(FEATURE_PROXY, "proxy");
def_feat!(FEATURE_BALANCE, "balance");
def_feat!(FEATURE_MIMALLOC, "mimalloc");
def_feat!(FEATURE_JEMALLOC, "jemalloc");
def_feat!(FEATURE_MULTI_THREAD, "multi-thread");
def_feat!(FEATURE_TRANSPORT, "transport");
def_feat!(FEATURE_BRUTAL_SHUTDOWN, "brutal-shutdown");

pub struct Features {
    pub mimalloc: bool,
    pub jemalloc: bool,
    pub multi_thread: bool,
    pub hook: bool,
    pub proxy: bool,
    pub balance: bool,
    pub transport: bool,
    pub brutal_shutdown: bool,
}

pub const FEATURES: Features = Features {
    mimalloc: FEATURE_MIMALLOC,
    jemalloc: FEATURE_JEMALLOC,
    multi_thread: FEATURE_MULTI_THREAD,
    hook: FEATURE_HOOK,
    proxy: FEATURE_PROXY,
    balance: FEATURE_BALANCE,
    transport: FEATURE_TRANSPORT,
    brutal_shutdown: FEATURE_BRUTAL_SHUTDOWN,
};

impl Display for Features {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        macro_rules! disp_feat {
            ($field: ident, $show: expr) => {
                if self.$field {
                    write!(f, "[{}]", $show)?;
                }
            };
        }

        disp_feat!(hook, "hook");
        disp_feat!(proxy, "proxy");
        disp_feat!(balance, "balance");
        disp_feat!(brutal_shutdown, "brutal");
        disp_feat!(transport, "transport");
        disp_feat!(multi_thread, "multi-thread");
        disp_feat!(mimalloc, "mimalloc");
        disp_feat!(jemalloc, "jemalloc");
        Ok(())
    }
}
