use std::fs;
use std::io::{Result, Error, ErrorKind};

use clap::ArgMatches;
use serde::{Serialize, Deserialize};

mod log;
pub use self::log::{LogLevel, LogConf};

mod dns;
pub use dns::{DnsMode, DnsProtocol, DnsConf};

mod net;
pub use net::{NetConf};

mod endpoint;
pub use endpoint::EndpointConf;

pub trait Config {
    type Output;

    fn build(self) -> Self::Output;

    fn rst_field(&mut self, other: &Self) -> &mut Self;

    fn take_field(&mut self, other: &Self) -> &mut Self;

    fn from_cmd_args(matches: &ArgMatches) -> Self;
}

#[derive(Debug, Default)]
pub struct CmdOverride {
    pub log: LogConf,
    pub dns: DnsConf,
    pub network: NetConf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FullConf {
    #[serde(default)]
    pub log: LogConf,

    #[serde(default)]
    pub dns: DnsConf,

    #[serde(default)]
    pub network: Option<NetConf>,

    pub endpoints: Vec<EndpointConf>,
}

impl FullConf {
    #[allow(unused)]
    pub fn new(
        log: LogConf,
        dns: DnsConf,
        network: Option<NetConf>,
        endpoints: Vec<EndpointConf>,
    ) -> Self {
        FullConf {
            log,
            dns,
            network,
            endpoints,
        }
    }

    pub fn from_conf_file(file: &str) -> Self {
        let conf = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("unable to open {}: {}", file, &e));
        match Self::from_conf_str(&conf) {
            Ok(x) => x,
            Err(e) => panic!("failed to parse {}: {}", file, &e),
        }
    }

    pub fn from_conf_str(conf: &str) -> Result<Self> {
        let toml_err = match toml::from_str(conf) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };
        let json_err = match serde_json::from_str(conf) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };

        Err(Error::new(
            ErrorKind::Other,
            format!(
                "parse as toml: {0}; parse as json: {1}",
                &toml_err, &json_err
            ),
        ))
    }

    pub fn add_endpoint(&mut self, endpoint: EndpointConf) -> &mut Self {
        self.endpoints.push(endpoint);
        self
    }

    pub fn apply_global_opts(&mut self, opts: CmdOverride) -> &mut Self {
        let CmdOverride {
            ref log,
            ref dns,
            ref network,
        } = opts;
        let global_network = self.network.unwrap_or_default();

        self.log.rst_field(log);
        self.dns.rst_field(dns);
        self.endpoints.iter_mut().for_each(|x| {
            x.network.take_field(&global_network).rst_field(network);
        });

        self
    }
}

#[macro_export]
macro_rules! rst {
    ($this: ident, $field: ident, $other: ident) => {
        let Self { $field, .. } = $other;
        if $field.is_some() {
            $this.$field = $field;
        }
    };
}

#[macro_export]
macro_rules! take {
    ($this: ident, $field: ident, $other: ident) => {
        let Self { $field, .. } = $other;
        if $this.$field.is_none() && $field.is_some() {
            $this.$field = $field;
        }
    };
}
