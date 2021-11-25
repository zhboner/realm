mod cmd;
mod dns;
mod conf;
mod utils;
mod relay;

use cfg_if::cfg_if;
use cmd::CmdInput;
use conf::FullConf;
use utils::Endpoint;

const VERSION: &str = "1.5.0-rc5";

cfg_if! {
    if #[cfg(all(feature = "mi-malloc"))] {
        use mimalloc::MiMalloc;
        #[global_allocator]
        static GLOBAL: MiMalloc = MiMalloc;
    }
}

cfg_if! {
    if #[cfg(all(feature = "jemalloc", not(target_env = "msvc")))] {
        use jemallocator::Jemalloc;
        #[global_allocator]
        static GLOBAL: Jemalloc = Jemalloc;
    }
}

fn main() {
    match cmd::scan() {
        CmdInput::Endpoint(ep) => execute(vec![ep]),
        CmdInput::Config(conf) => start_from_conf(conf),
        CmdInput::None => {}
    }
}

fn start_from_conf(conf: String) {
    let FullConf {
        log: log_conf,
        dns: dns_conf,
        endpoints: eps_conf,
    } = FullConf::from_config_file(&conf);

    setup_log(log_conf);
    setup_dns(dns_conf);

    let eps: Vec<Endpoint> = eps_conf
        .into_iter()
        .map(|epc| {
            let ep = epc.build();
            log::info!("inited: {}", &ep);
            ep
        })
        .collect();

    execute(eps);
}

fn setup_log(conf: conf::LogConf) {
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
            .unwrap_or_else(|e| panic!("failed to setup logger: {}", &e))
    }
}

#[allow(unused_variables)]
fn setup_dns(dns: conf::CompatibeDnsConf) {
    #[cfg(feature = "trust-dns")]
    {
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
}

fn execute(eps: Vec<Endpoint>) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(relay::run(eps))
}
