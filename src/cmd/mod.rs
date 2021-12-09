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
        Arg::new("use_udp")
            .short('u')
            .long("udp")
            .about("enable udp forward")
            .display_order(0),
        Arg::new("fast_open")
            .short('f')
            .long("tfo")
            .about("enable tcp fast open")
            .display_order(1),
        Arg::new("zero_copy")
            .short('z')
            .long("splice")
            .about("enable tcp zero copy")
            .display_order(2),
        Arg::new("daemon")
            .short('d')
            .long("daemon")
            .about("run as a unix daemon")
            .display_order(3),
    ])
}

fn add_options(app: App) -> App {
    app.help_heading("OPTIONS").args(&[
        Arg::new("help")
            .short('h')
            .long("help")
            .about("show help")
            .display_order(0),
        Arg::new("version")
            .short('v')
            .long("version")
            .about("show version")
            .display_order(1),
        Arg::new("config")
            .short('c')
            .long("config")
            .about("use config file")
            .value_name("path")
            .takes_value(true)
            .display_order(2),
        Arg::new("local")
            .short('l')
            .long("listen")
            .about("listen address")
            .value_name("addr")
            .takes_value(true)
            .display_order(3),
        Arg::new("remote")
            .short('r')
            .long("remote")
            .about("remote address")
            .value_name("addr")
            .takes_value(true)
            .display_order(4),
        Arg::new("through")
            .short('x')
            .long("through")
            .about("send through ip or address")
            .value_name("addr")
            .takes_value(true)
            .display_order(5),
        Arg::new("tcp_timeout")
            .long("tcp-timeout")
            .about("set timeout value for tcp")
            .value_name("second")
            .takes_value(true)
            .display_order(6),
        Arg::new("udp_timeout")
            .long("udp-timeout")
            .about("set timeout value for udp")
            .value_name("second")
            .takes_value(true)
            .display_order(7),
    ])
}

fn add_global_options(app: App) -> App {
    app.help_heading("GLOBAL OPTIONS").args(&[
        Arg::new("log_level")
            .long("log-level")
            .about("override log level")
            .value_name("level")
            .takes_value(true)
            .display_order(0),
        Arg::new("log_output")
            .long("log-output")
            .about("override log output")
            .value_name("path")
            .takes_value(true)
            .display_order(1),
        Arg::new("dns_mode")
            .long("dns-mode")
            .about("override dns mode")
            .value_name("mode")
            .takes_value(true)
            .display_order(2),
        Arg::new("dns_protocol")
            .long("dns-protocol")
            .about("override dns protocol")
            .value_name("protocol")
            .takes_value(true)
            .display_order(3),
        Arg::new("dns_servers")
            .long("dns-servers")
            .about("override dns servers")
            .value_name("servers")
            .takes_value(true)
            .display_order(4),
    ])
}

pub fn scan() -> CmdInput {
    let version = format!("{} {}", VERSION, FEATURES);
    let app = App::new("Realm")
        .about("A high efficiency relay tool")
        .version(version.as_str())
        .license("MIT");

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
