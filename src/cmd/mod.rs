use clap::{App, AppSettings};
use clap::{Arg, ArgMatches};

use crate::conf::CmdOverride;
use crate::conf::EndpointConf;
use crate::conf::{Config, LogConf, DnsConf, NetConf};

use super::VERSION;
use crate::utils::FEATURES;

pub enum CmdInput {
    Config(String, CmdOverride),
    Endpoint(EndpointConf, CmdOverride),
    None,
}

fn add_flags(app: App) -> App {
    app.help_heading("FLAGS").args(&[
        Arg::new("help")
            .short('h')
            .long("help")
            .help("show help")
            .display_order(0),
        Arg::new("version")
            .short('v')
            .long("version")
            .help("show version")
            .display_order(1),
        Arg::new("daemon")
            .short('d')
            .long("daemon")
            .help("run as a unix daemon")
            .display_order(2),
        Arg::new("use_udp")
            .short('u')
            .long("udp")
            .help("force enable udp forward")
            .display_order(3),
        Arg::new("fast_open")
            .short('f')
            .long("tfo")
            .help("force enable tcp fast open")
            .display_order(4),
        Arg::new("zero_copy")
            .short('z')
            .long("splice")
            .help("force enable tcp zero copy")
            .display_order(5),
    ])
}

fn add_options(app: App) -> App {
    app.help_heading("OPTIONS").args(&[
        Arg::new("nofile")
            .short('n')
            .long("nofile")
            .help("set nofile limit")
            .value_name("limit")
            .takes_value(true)
            .display_order(0),
        Arg::new("config")
            .short('c')
            .long("config")
            .help("use config file")
            .value_name("path")
            .takes_value(true)
            .display_order(1),
        Arg::new("local")
            .short('l')
            .long("listen")
            .help("listen address")
            .value_name("addr")
            .takes_value(true)
            .display_order(2),
        Arg::new("remote")
            .short('r')
            .long("remote")
            .help("remote address")
            .value_name("addr")
            .takes_value(true)
            .display_order(3),
        Arg::new("through")
            .short('x')
            .long("through")
            .help("send through ip or address")
            .value_name("addr")
            .takes_value(true)
            .display_order(4),
    ])
}

fn add_global_options(app: App) -> App {
    // log
    let app = app.help_heading("LOG OPTIONS").args(&[
        Arg::new("log_level")
            .long("log-level")
            .help("override log level")
            .value_name("level")
            .takes_value(true)
            .display_order(0),
        Arg::new("log_output")
            .long("log-output")
            .help("override log output")
            .value_name("path")
            .takes_value(true)
            .display_order(1),
    ]);

    // dns
    let app = app.help_heading("DNS OPTIONS").args(&[
        Arg::new("dns_mode")
            .long("dns-mode")
            .help("override dns mode")
            .value_name("mode")
            .takes_value(true)
            .display_order(0),
        Arg::new("dns_min_ttl")
            .long("dns-min-ttl")
            .help("override dns min ttl")
            .value_name("second")
            .takes_value(true)
            .display_order(1),
        Arg::new("dns_max_ttl")
            .long("dns-max-ttl")
            .help("override dns max ttl")
            .value_name("second")
            .takes_value(true)
            .display_order(2),
        Arg::new("dns_cache_size")
            .long("dns-cache-size")
            .help("override dns cache size")
            .value_name("number")
            .takes_value(true)
            .display_order(3),
        Arg::new("dns_protocol")
            .long("dns-protocol")
            .help("override dns protocol")
            .value_name("protocol")
            .takes_value(true)
            .display_order(4),
        Arg::new("dns_servers")
            .long("dns-servers")
            .help("override dns servers")
            .value_name("servers")
            .takes_value(true)
            .display_order(5),
    ]);

    // proxy-protocol belogs to network
    let app = app.help_heading("PROXY OPTIONS").args([
        Arg::new("send_proxy")
            .long("send-proxy")
            .help("send haproxy proxy protocol")
            .display_order(0),
        Arg::new("accept_proxy")
            .long("accept-proxy")
            .help("accept haproxy proxy protocol")
            .display_order(1),
        Arg::new("send_proxy_version")
            .long("send-proxy-version")
            .help("haproxy proxy protocol version")
            .value_name("version")
            .takes_value(true)
            .display_order(2),
    ]);

    // timeout belogs to network
    let app = app.help_heading("TIMEOUT OPTIONS").args([
        Arg::new("tcp_timeout")
            .long("tcp-timeout")
            .help("override tcp timeout")
            .value_name("second")
            .takes_value(true)
            .display_order(0),
        Arg::new("udp_timeout")
            .long("udp-timeout")
            .help("override udp timeout")
            .value_name("second")
            .takes_value(true)
            .display_order(1),
    ]);

    app
}

pub fn scan() -> CmdInput {
    let version = format!("{} {}", VERSION, FEATURES);
    let app = App::new("Realm")
        .about("A high efficiency relay tool")
        .version(version.as_str());

    let app = app
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableVersionFlag)
        .setting(
            AppSettings::DisableHelpFlag | AppSettings::DisableHelpSubcommand,
        )
        .override_usage("realm [FLAGS] [OPTIONS]");

    let app = add_flags(app);
    let app = add_options(app);
    let app = add_global_options(app);

    let mut xapp = app.clone();
    let matches = app.get_matches();

    if matches.is_present("help") {
        xapp.print_help().unwrap();
        return CmdInput::None;
    }

    if matches.is_present("version") {
        print!("{}", xapp.render_version());
        return CmdInput::None;
    }

    parse_matches(matches)
}

fn parse_matches(matches: ArgMatches) -> CmdInput {
    #[cfg(unix)]
    if matches.is_present("daemon") {
        crate::utils::daemonize();
    }

    #[cfg(all(unix, not(target_os = "android")))]
    {
        use crate::utils::get_nofile_limit;
        use crate::utils::set_nofile_limit;

        // get
        if let Some((soft, hard)) = get_nofile_limit() {
            println!("nofile limit: soft={}, hard={}", soft, hard);
        }

        // set
        if let Some(nofile) = matches.value_of("nofile") {
            if let Ok(nofile) = nofile.parse::<u64>() {
                set_nofile_limit(nofile);
            } else {
                eprintln!("invalid nofile value: {}", nofile);
            }
        }
    }

    let opts = parse_global_opts(&matches);

    if let Some(config) = matches.value_of("config") {
        return CmdInput::Config(String::from(config), opts);
    }

    if matches.value_of("local").is_some()
        && matches.value_of("remote").is_some()
    {
        let ep = EndpointConf::from_cmd_args(&matches);
        return CmdInput::Endpoint(ep, opts);
    }

    CmdInput::None
}

fn parse_global_opts(matches: &ArgMatches) -> CmdOverride {
    let log = LogConf::from_cmd_args(matches);
    let dns = DnsConf::from_cmd_args(matches);
    let network = NetConf::from_cmd_args(matches);
    CmdOverride { log, dns, network }
}
