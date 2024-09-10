use std::fs;
use std::io::{Result, Error, ErrorKind};
use walkdir::WalkDir;

use clap::ArgMatches;
use serde::{Serialize, Deserialize};

mod log;
pub use self::log::{LogLevel, LogConf};

mod dns;
pub use dns::{DnsMode, DnsProtocol, DnsConf};

mod net;
pub use net::{NetConf, NetInfo};

mod endpoint;
pub use endpoint::{EndpointConf, EndpointInfo};

mod legacy;
pub use legacy::LegacyConf;

/// Conig Architecture
/// cmd | file => LogConf => { level, output }
/// cmd | file => DnsConf => { resolve cinfig, opts }
/// cmd | file => NetConf
///                      \
/// cmd | file => EndpointConf => { [local, remote, conn_opts] }

pub trait Config {
    type Output;

    fn is_empty(&self) -> bool;

    fn build(self) -> Self::Output;

    // override self if other not empty
    // e.g.: cmd argument overrides global and local option
    fn rst_field(&mut self, other: &Self) -> &mut Self;

    // take other only if self empty & other not empty
    // e.g.: local field takes global option
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
    #[serde(skip_serializing_if = "Config::is_empty")]
    pub log: LogConf,

    #[serde(default)]
    #[serde(skip_serializing_if = "Config::is_empty")]
    pub dns: DnsConf,

    #[serde(default)]
    #[serde(skip_serializing_if = "Config::is_empty")]
    pub network: NetConf,

    pub endpoints: Vec<EndpointConf>,
}

impl FullConf {
    #[allow(unused)]
    pub fn new(log: LogConf, dns: DnsConf, network: NetConf, endpoints: Vec<EndpointConf>) -> Self {
        FullConf {
            log,
            dns,
            network,
            endpoints,
        }
    }

    pub fn from_conf_file(file: &str) -> Self {
        let mtd = fs::metadata(file);
        if mtd.is_err() {
            eprintln!("failed to open {}: {}", file, mtd.err().unwrap());
            std::process::exit(0)
        }

        let attrs = mtd.unwrap();
        if attrs.is_file() {
            let conf = fs::read_to_string(file).unwrap_or_else(|e| panic!("unable to open {}: {}", file, &e));
            match Self::from_conf_str(&conf) {
                Ok(x) => return x,
                Err(e) => panic!("failed to parse {}: {}", file, &e),
            }
        }

        let mut conf = FullConf::default();
        for entry in WalkDir::new(file)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.file_name().to_string_lossy().ends_with(".toml") || e.file_name().to_string_lossy().ends_with(".json")
            })
        {
            let mut conf_part = FullConf::default();
            let conf_frag = fs::read_to_string(entry.path())
                .unwrap_or_else(|e| panic!("unable to open {:#?}: {}", entry.path(), &e));

            let f_name = entry.file_name().to_string_lossy();
            if f_name.ends_with(".json") {
                conf_part = serde_json::from_str(conf_frag.as_str())
                    .unwrap_or_else(|e| panic!("failed to parse {:#?}: {}", entry.path(), &e));
            } else if f_name.ends_with(".toml") {
                conf_part = toml::from_str(conf_frag.as_str())
                    .unwrap_or_else(|e| panic!("failed to parse {:#?}: {}", entry.path(), &e));
            }

            if !conf_part.dns.is_empty() {
                conf.dns.take_field(&conf_part.dns);
            }
            if !conf_part.log.is_empty() {
                conf.log.take_field(&conf_part.log);
            }
            if !conf_part.network.is_empty() {
                conf.network.take_field(&conf_part.network);
            }
            if !conf_part.endpoints.is_empty() {
                for ep in conf_part.endpoints {
                    conf.endpoints.push(ep);
                }
            }
        }

        let conf_str = toml::to_string(&conf).unwrap();
        match Self::from_conf_str(&conf_str.to_string()) {
            Ok(x) => x,
            Err(e) => panic!("failed to parse {}: {}", file, &e),
        }
    }

    pub fn from_conf_str(s: &str) -> Result<Self> {
        let toml_err = match toml::from_str(s) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };

        let json_err = match serde_json::from_str(s) {
            Ok(x) => return Ok(x),
            Err(e) => e,
        };

        // to be compatible with old version
        let legacy_err = match serde_json::from_str::<LegacyConf>(s) {
            Ok(x) => {
                eprintln!("attention: you are using a legacy config file!");
                return Ok(x.into());
            }
            Err(e) => e,
        };

        Err(Error::new(
            ErrorKind::Other,
            format!(
                "parse as toml: {0}; parse as json: {1}; parse as legacy: {2}",
                toml_err, json_err, legacy_err
            ),
        ))
    }

    pub fn add_endpoint(&mut self, endpoint: EndpointConf) -> &mut Self {
        self.endpoints.push(endpoint);
        self
    }

    // override
    pub fn apply_cmd_opts(&mut self, opts: CmdOverride) -> &mut Self {
        let CmdOverride {
            ref log,
            ref dns,
            ref network,
        } = opts;

        self.log.rst_field(log);
        self.dns.rst_field(dns);
        self.endpoints.iter_mut().for_each(|x| {
            x.network.rst_field(network);
        });

        self
    }

    // take inner global opts
    pub fn apply_global_opts(&mut self) -> &mut Self {
        self.endpoints.iter_mut().for_each(|x| {
            x.network.take_field(&self.network);
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

#[macro_export]
macro_rules! empty {
    ( $this: expr => $( $field: ident ),* ) => {{
        let mut res = true;
        $(
            res = res && $this.$field.is_none();
        )*
        res
    }};
    ( $( $value: expr ),* ) => {{
        let mut res = true;
        $(
            res = res && $value.is_none();
        )*
        res
    }}
}
