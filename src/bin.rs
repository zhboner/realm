use std::env;
use cfg_if::cfg_if;

use realm::cmd;
use realm::conf::{Config, FullConf, LogConf, DnsConf, EndpointInfo};
use realm::ENV_CONFIG;

cfg_if! {
    if #[cfg(feature = "mi-malloc")] {
        use mimalloc::MiMalloc;
        #[global_allocator]
        static GLOBAL: MiMalloc = MiMalloc;
    } else if #[cfg(all(feature = "jemalloc", not(target_env = "msvc")))] {
        use jemallocator::Jemalloc;
        #[global_allocator]
        static GLOBAL: Jemalloc = Jemalloc;
    } else if #[cfg(all(feature = "page-alloc", unix))] {
        use mmap_allocator::MmapAllocator;
        #[global_allocator]
        static GLOBAL: MmapAllocator = MmapAllocator::new();
    }
}

fn main() {
    let conf = 'blk: {
        if let Ok(conf_str) = env::var(ENV_CONFIG) {
            if let Ok(conf) = FullConf::from_conf_str(&conf_str) {
                break 'blk conf;
            }
        };

        use cmd::CmdInput;
        match cmd::scan() {
            CmdInput::Endpoint(ep, opts) => {
                let mut conf = FullConf::default();
                conf.add_endpoint(ep).apply_global_opts().apply_cmd_opts(opts);
                conf
            }
            CmdInput::Config(conf, opts) => {
                let mut conf = FullConf::from_conf_file(&conf);
                conf.apply_global_opts().apply_cmd_opts(opts);
                conf
            }
            CmdInput::None => std::process::exit(0),
        }
    };

    start_from_conf(conf);
}

fn start_from_conf(full: FullConf) {
    let FullConf {
        log: log_conf,
        dns: dns_conf,
        endpoints: endpoints_conf,
        ..
    } = full;

    setup_log(log_conf);
    setup_dns(dns_conf);

    let endpoints: Vec<EndpointInfo> = endpoints_conf
        .into_iter()
        .map(Config::build)
        .inspect(|x| println!("inited: {}", x.endpoint))
        .collect();

    execute(endpoints);
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

fn setup_dns(dns: DnsConf) {
    println!("dns: {}", &dns);

    let (conf, opts) = dns.build();
    realm::core::dns::build_lazy(conf, opts);
}

fn execute(eps: Vec<EndpointInfo>) {
    #[cfg(feature = "multi-thread")]
    {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run(eps))
    }

    #[cfg(not(feature = "multi-thread"))]
    {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run(eps))
    }
}

async fn run(endpoints: Vec<EndpointInfo>) {
    use realm::core::tcp::run_tcp;
    use realm::core::udp::run_udp;
    use futures::future::join_all;

    let mut workers = Vec::with_capacity(2 * endpoints.len());

    for EndpointInfo {
        endpoint,
        no_tcp,
        use_udp,
    } in endpoints
    {
        if use_udp {
            workers.push(tokio::spawn(run_udp(endpoint.clone())));
        }

        if !no_tcp {
            workers.push(tokio::spawn(run_tcp(endpoint)));
        }
    }

    workers.shrink_to_fit();

    join_all(workers).await;
}
