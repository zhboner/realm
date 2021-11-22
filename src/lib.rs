use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "realm", about = "A high efficiency proxy tool.")]
pub struct Cli {
    #[structopt(short = "l", long = "local")]
    pub client: Option<String>,

    #[structopt(short = "r", long = "remote")]
    pub remote: Option<String>,

    #[structopt(
        short = "c",
        long = "config",
        parse(from_os_str),
        name = "Optional config file",
        conflicts_with_all = &["client", "remote"],
        required_unless_all = &["client", "remote"]
    )]
    pub config_file: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug)]

pub struct RelayConfig {
    pub listening_address: String,
    pub listening_port: String,
    pub remote_address: String,
    pub remote_port: String,
}

impl Default for RelayConfig {
    fn default() -> RelayConfig {
        RelayConfig {
            listening_address: String::from("0.0.0.0"),
            listening_port: String::from("1080"),
            remote_address: String::from("127.0.0.1"),
            remote_port: String::from("8080"),
        }
    }
}

impl RelayConfig {
    fn new(ld: String, lp: String, rd: String, rp: String) -> RelayConfig {
        RelayConfig {
            listening_address: ld,
            listening_port: lp,
            remote_address: rd,
            remote_port: rp,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConfigFile {
    pub listening_addresses: Vec<String>,
    pub listening_ports: Vec<String>,
    pub remote_addresses: Vec<String>,
    pub remote_ports: Vec<String>,
}

/// Another config file
/// This config file does not support port range.
/// # Example
/// ```json
/// {
///     "relays": [
///         {
///             "listen": "127.0.0.1:1080",//listening address must have address and port
///             "remote": "127.0.0.1:8080"//remote address must have address and port
///         },
///         {
///             "listen": "127.0.0.1:2080",
///             "remote": "127.0.0.1:8080"
///         }
///     ]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct AnotherConfigFile {
    pub relays: Vec<Relay>,
}
impl AnotherConfigFile {
    fn to_relay_config(&self) -> Vec<RelayConfig> {
        return self
            .relays
            .iter()
            .map(|relay| relay.into_relayconfig())
            .collect::<Vec<RelayConfig>>();
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Relay {
    pub listen: String,
    pub remote: String,
}

impl Relay {
    fn into_relayconfig(&self) -> RelayConfig {
        let (listening_address, listening_port) = self
            .listen
            .rsplit_once(":")
            .expect("Invalid listen address");
        let (remote_address, remote_port) = self
            .remote
            .rsplit_once(":")
            .expect("Invalid remote address");
        RelayConfig::new(
            listening_address.to_string(),
            listening_port.to_string(),
            remote_address.to_string(),
            remote_port.to_string(),
        )
    }
}
pub fn parse_arguments() -> Vec<RelayConfig> {
    let input = Cli::from_args();
    let path = input.config_file;
    if let Some(path) = path {
        return load_config(path);
    }

    let client = match input.client {
        Some(client) => client,
        None => panic!("No listening socket"),
    };

    let remote = match input.remote {
        Some(remote) => remote,
        None => panic!("No listening socket"),
    };

    let client_parse: Vec<&str> = client
        .rsplitn(2, ":")
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .collect();
    if client_parse.len() != 2 {
        panic!("client address is incorrect!");
    }
    let listening_address = String::from_str(client_parse[0]).unwrap();

    let remote_parse: Vec<&str> = remote
        .rsplitn(2, ":")
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .collect();
    if remote_parse.len() != 2 {
        panic!("remote address is incorrect!");
    }

    vec![RelayConfig {
        listening_address: if listening_address == "" {
            String::from("0.0.0.0")
        } else {
            listening_address
        },
        listening_port: String::from_str(client_parse[1]).unwrap(),
        remote_address: String::from_str(remote_parse[0]).unwrap(),
        remote_port: String::from_str(remote_parse[1]).unwrap(),
    }]
}

fn ports2individuals(ports: Vec<String>) -> Vec<u16> {
    let mut output = vec![];

    // Convert port ranges to individual ports
    for lp in ports {
        if lp.find("-").is_none() {
            output.push(lp.parse::<u16>().unwrap())
        } else {
            let ints: Vec<&str> = lp.split("-").collect();
            if ints.len() != 2 {
                panic!("Invalid range")
            }
            let st = ints[0].parse::<u16>().unwrap();
            let end = ints[1].parse::<u16>().unwrap();
            if st > end {
                panic!("Invalid range")
            }

            for i in st..=end {
                output.push(i);
            }
        }
    }
    output
}

pub fn load_config(p: PathBuf) -> Vec<RelayConfig> {
    // let path = Path::new(&p);
    // let display = p.display();

    let config_file_string = read_to_string(&p).unwrap();
    let config = serde_json::from_str::<ConfigFile>(config_file_string.clone().as_str());
    let config = match config {
        Err(_e) => {
            println!("AnotherConfigFile format may help you.");
            let config = serde_json::from_str::<AnotherConfigFile>(config_file_string.as_str());
            match config {
                Err(e) => panic!("Could not parse config file {}: {}", p.display(), e),
                Ok(config) => {
                    println!("{} contains {:#?}", p.display(), config.to_relay_config());
                    return config.to_relay_config();
                }
            }
        }
        Ok(config) => config,
    };
    let listening_ports = ports2individuals(config.listening_ports);
    let remote_ports = ports2individuals(config.remote_ports);

    // if listening_ports.len() != remote_ports.len() {
    //     panic!("Unmatched number of listening and remot ports")
    // }

    // if config.listening_addresses.len() != 1
    //     && config.listening_addresses.len() != listening_ports.len()
    // {
    //     panic!("Unmatched listening address and ports")
    // }

    // if config.remote_addresses.len() != 1 && config.remote_addresses.len() != remote_ports.len() {
    //     panic!("Unmatched remote address and ports")
    // }

    let mut relay_configs = vec![];
    let total = listening_ports.len();

    for i in 0..total {
        let ld = match config.listening_addresses.get(i) {
            Some(ld) => ld,
            None => &config.listening_addresses[0],
        };

        let rd = match config.remote_addresses.get(i) {
            Some(rd) => rd,
            None => &config.remote_addresses[0],
        };

        let rp = match remote_ports.get(i) {
            Some(rp) => rp,
            None => &remote_ports[0],
        };

        let lp = match listening_ports.get(i) {
            Some(lp) => lp,
            None => &listening_ports[0],
        };

        relay_configs.push(RelayConfig::new(
            ld.to_string(),
            lp.to_string(),
            rd.to_string(),
            rp.to_string(),
        ))
    }
    relay_configs
}
pub fn load_config_alternate(p: PathBuf) -> Vec<RelayConfig> {
    let f = match File::open(&p) {
        Err(e) => panic!("Could not open config file {}: {}", p.display(), e),
        Ok(f) => f,
    };
    let reader = BufReader::new(f);
    let config: AnotherConfigFile = serde_json::from_reader(reader).unwrap();
    config.to_relay_config()
}
