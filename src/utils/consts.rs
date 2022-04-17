use std::fmt::{Display, Formatter};

// default logfile
pub const DEFAULT_LOG_FILE: &str = "stdout";

// default timeout
pub const TCP_TIMEOUT: usize = 300;
pub const UDP_TIMEOUT: usize = 30;

// default haproxy proxy-protocol version
pub const PROXY_PROTOCOL_VERSION: usize = 2;

// default haproxy proxy-protocol version
pub const PROXY_PROTOCOL_TIMEOUT: usize = 5;

// https://github.com/rust-lang/rust/blob/master/library/std/src/sys_common/io.rs#L1
pub const DEFAULT_BUF_SIZE: usize = if cfg!(target_os = "espidf") {
    512
} else {
    8 * 1024
};

// Since Linux 2.6.11, the pipe capacity is 16 pages
#[cfg(all(target_os = "linux", feature = "zero-copy"))]
pub const DEFAULT_PIPE_CAP: usize = 16 * 4096;

#[cfg(all(target_os = "linux", feature = "zero-copy"))]
pub static mut CUSTOM_PIPE_CAP: usize = DEFAULT_PIPE_CAP;

// features
macro_rules! def_feat {
    ($fet: ident, $name: expr) => {
        pub const $fet: bool = if cfg!(feature = $name) { true } else { false };
    };
}

def_feat!(FEATURE_UDP, "udp");
def_feat!(FEATURE_TFO, "tfo");
def_feat!(FEATURE_ZERO_COPY, "zero-copy");
def_feat!(FEATURE_TRUST_DNS, "trust-dns");
def_feat!(FEATURE_PROXY_PROTOCOL, "proxy-protocol");
def_feat!(FEATURE_MIMALLOC, "mi-malloc");
def_feat!(FEATURE_JEMALLOC, "jemalloc");
def_feat!(FEATURE_MULTI_THREAD, "multi-thread");
def_feat!(FEATURE_TRANSPORT, "transport");

pub struct Features {
    pub udp: bool,
    pub tfo: bool,
    pub zero_copy: bool,
    pub trust_dns: bool,
    pub mimalloc: bool,
    pub jemalloc: bool,
    pub multi_thread: bool,
    pub proxy_protocol: bool,
    pub transport: bool,
}

pub const FEATURES: Features = Features {
    udp: FEATURE_UDP,
    tfo: FEATURE_TFO,
    zero_copy: FEATURE_ZERO_COPY,
    trust_dns: FEATURE_TRUST_DNS,
    mimalloc: FEATURE_MIMALLOC,
    jemalloc: FEATURE_JEMALLOC,
    multi_thread: FEATURE_MULTI_THREAD,
    proxy_protocol: FEATURE_PROXY_PROTOCOL,
    transport: FEATURE_TRANSPORT,
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
        disp_feat!(udp, "udp");
        disp_feat!(tfo, "tfo");
        disp_feat!(zero_copy, "zero-copy");
        disp_feat!(trust_dns, "trust-dns");
        disp_feat!(proxy_protocol, "proxy-protocol");
        disp_feat!(transport, "transport");
        disp_feat!(multi_thread, "multi-thread");
        disp_feat!(mimalloc, "mimalloc");
        disp_feat!(jemalloc, "jemalloc");
        Ok(())
    }
}
