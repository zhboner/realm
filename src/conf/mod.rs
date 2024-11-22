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

#[allow(clippy::too_long_first_doc_paragraph)]
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
        let mtd = fs::metadata(file).unwrap_or_else(|e| panic!("failed to open {}: {}", file, e));

        if mtd.is_file() {
            let conf = fs::read_to_string(file).unwrap_or_else(|e| panic!("failed to open {}: {}", file, e));
            match Self::from_conf_str(&conf) {
                Ok(x) => return x,
                Err(e) => panic!("failed to parse {}: {}", file, &e),
            }
        }

        let mut full_conf = FullConf::default();
        for entry in WalkDir::new(file)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| e.file_name().to_str().is_some_and(|x| !x.starts_with('.')))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|s| s == "toml" || s == "json"))
        {
            let conf_part = fs::read_to_string(entry.path())
                .unwrap_or_else(|e| panic!("failed to open {}: {}", entry.path().to_string_lossy(), e));

            let conf_part = Self::from_conf_str(&conf_part)
                .unwrap_or_else(|e| panic!("failed to parse {}: {}", entry.path().to_string_lossy(), e));
            full_conf.take_fields(conf_part);
        }
        full_conf
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

    fn take_fields(&mut self, other: Self) {
        self.log.take_field(&other.log);
        self.dns.take_field(&other.dns);
        self.network.take_field(&other.network);
        self.endpoints.extend(other.endpoints);
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
