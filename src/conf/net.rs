use serde::{Serialize, Deserialize};
use super::Config;
use crate::utils::ConnectOpts;
use crate::utils::{TCP_TIMEOUT, UDP_TIMEOUT};

#[derive(Serialize, Debug, Deserialize, Clone, Copy, Default)]
pub struct NetConf {
    #[serde(default)]
    pub use_udp: Option<bool>,

    #[serde(default)]
    pub fast_open: Option<bool>,

    #[serde(default)]
    pub zero_copy: Option<bool>,

    #[serde(default)]
    pub tcp_timeout: Option<u64>,

    #[serde(default)]
    pub udp_timeout: Option<u64>,
}

impl Config for NetConf {
    type Output = ConnectOpts;

    fn build(self) -> Self::Output {
        macro_rules! unbox {
            ($field: ident) => {
                self.$field.unwrap_or_default()
            };
            ($field: ident, $value: expr) => {
                self.$field.unwrap_or($value)
            };
        }

        let use_udp = unbox!(use_udp);
        let fast_open = unbox!(fast_open);
        let zero_copy = unbox!(zero_copy);
        let tcp_timeout = unbox!(tcp_timeout, TCP_TIMEOUT);
        let udp_timeout = unbox!(udp_timeout, UDP_TIMEOUT);

        ConnectOpts {
            use_udp,
            fast_open,
            zero_copy,
            tcp_timeout,
            udp_timeout,
            send_through: None,
        }
    }

    fn rst_field(&mut self, other: &Self) -> &mut Self {
        use crate::rst;
        let other = *other;

        rst!(self, use_udp, other);
        rst!(self, fast_open, other);
        rst!(self, zero_copy, other);
        rst!(self, tcp_timeout, other);
        rst!(self, udp_timeout, other);
        self
    }

    fn take_field(&mut self, other: &Self) -> &mut Self {
        use crate::take;
        let other = *other;

        take!(self, use_udp, other);
        take!(self, fast_open, other);
        take!(self, zero_copy, other);
        take!(self, tcp_timeout, other);
        take!(self, udp_timeout, other);
        self
    }

    fn from_cmd_args(matches: &clap::ArgMatches) -> Self {
        let use_udp = matches.is_present("use_udp");
        let fast_open = matches.is_present("fast_open");
        let zero_copy = matches.is_present("zero_copy");

        let tcp_timeout = matches
            .value_of("tcp_timeout")
            .map(|x| x.parse::<u64>().unwrap());
        let udp_timeout = matches
            .value_of("udp_timeout")
            .map(|x| x.parse::<u64>().unwrap());

        const fn bool_to_opt(b: bool) -> Option<bool> {
            if b {
                Some(true)
            } else {
                None
            }
        }

        Self {
            use_udp: bool_to_opt(use_udp),
            fast_open: bool_to_opt(fast_open),
            zero_copy: bool_to_opt(zero_copy),
            tcp_timeout,
            udp_timeout,
        }
    }
}
