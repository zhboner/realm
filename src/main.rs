mod cmd;
mod dns;
mod conf;
mod utils;
mod relay;

use cmd::CmdInput;
use conf::FullConf;
use utils::Endpoint;

const VERSION: &str = "1.5.0-rc5";

fn main() {
    match cmd::scan() {
        CmdInput::Endpoint(ep) => start_from_cmd(ep),
        CmdInput::Config(conf) => start_from_conf(conf),
        CmdInput::Navigate => cmd::run_navigator(),
        CmdInput::None => {}
    }
}

fn start_from_cmd(ep: Endpoint) {
    run_relay(vec![ep])
}

fn start_from_conf(conf: String) {
    let conf = FullConf::from_config_file(&conf);
    #[cfg(feature = "trust-dns")]
    {
        let FullConf { dns, .. } = conf;
        setup_dns(dns);
    }

    let eps: Vec<Endpoint> = conf
        .endpoints
        .into_iter()
        .map(|epc| {
            let ep = epc.build();
            log::info!("inited: {}", &ep);
            ep
        })
        .collect();

    run_relay(eps);
}

fn setup_logger(conf: conf::LogConf) {
    #[cfg(feature = "x-debug")]
    env_logger::init();

    #[cfg(not(feature = "x-debug"))]
    {
        let (level, output) = conf.into();
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(level)
            .chain(output)
            .apply()
            .unwrap_or_else(|ref e| panic!("failed to setup logger: {}", e))
    }
}

#[cfg(feature = "trust-dns")]
fn setup_dns(dns: conf::CompatibeDnsConf) {
    use conf::CompatibeDnsConf::*;
    match dns {
        Dns(conf) => {
            let (conf, opts) = conf.into();
            dns::configure(Some(conf), Some(opts));
        }
        DnsMode(mode) => dns::configure(Option::None, Some(mode.into())),
        None => (),
    }
    dns::build();
}

fn run_relay(eps: Vec<Endpoint>) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(relay::run(eps))
}
