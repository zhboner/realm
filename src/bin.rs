use std::env;
use cfg_if::cfg_if;

use realm::cmd;
use realm::dns;
use realm::conf::{Config, FullConf, LogConf, DnsConf};
use realm::utils::Endpoint;
use realm::relay;
use realm::ENV_CONFIG;

cfg_if! {
    if #[cfg(all(feature = "mi-malloc"))] {
        use mimalloc::MiMalloc;
        #[global_allocator]
        static GLOBAL: MiMalloc = MiMalloc;
    } else if #[cfg(all(feature = "jemalloc", not(target_env = "msvc")))] {
        use jemallocator::Jemalloc;
        #[global_allocator]
        static GLOBAL: Jemalloc = Jemalloc;
    }
}

fn main() {
    let conf = (|| {
        if let Ok(conf_str) = env::var(ENV_CONFIG) {
            if let Ok(conf) = FullConf::from_conf_str(&conf_str) {
                return conf;
            }
        };

        use cmd::CmdInput;
        match cmd::scan() {
            CmdInput::Endpoint(ep, opts) => {
                let mut conf = FullConf::default();
                conf.add_endpoint(ep)
                    .apply_global_opts()
                    .apply_cmd_opts(opts);
                conf
            }
            CmdInput::Config(conf, opts) => {
                let mut conf = FullConf::from_conf_file(&conf);
                conf.apply_global_opts().apply_cmd_opts(opts);
                conf
            }
            CmdInput::None => std::process::exit(0),
        }
    })();

    start_from_conf(conf);
}

fn start_from_conf(full: FullConf) {
    let FullConf {
        log: log_conf,
        dns: dns_conf,
        endpoints: eps_conf,
        ..
    } = full;

    setup_log(log_conf);
    setup_dns(dns_conf);

    let eps: Vec<Endpoint> = eps_conf
        .into_iter()
        .map(|epc| epc.build())
        .inspect(|x| println!("inited: {}", &x))
        .collect();

    execute(eps);
}

fn setup_log(log: LogConf) {
    println!("log: {}", &log);

    let (level, output) = log.build();
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}]{}",
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

#[allow(unused_variables)]
fn setup_dns(dns: DnsConf) {
    println!("dns: {}", &dns);

    #[cfg(feature = "trust-dns")]
    {
        let (conf, opts) = dns.build();
        dns::configure(conf, opts);
        dns::build();
    }
}

fn execute(eps: Vec<Endpoint>) {
    #[cfg(feature = "multi-thread")]
    {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(relay::run(eps))
    }

    #[cfg(not(feature = "multi-thread"))]
    {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(relay::run(eps))
    }
}
