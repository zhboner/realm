use clap::{App, Arg, ArgMatches, AppSettings};

use super::Endpoint;
use super::VERSION;
use crate::utils::FEATURES;
use crate::utils::TCP_TIMEOUT;
use crate::utils::UDP_TIMEOUT;

pub enum CmdInput {
    Config(String),
    Endpoint(Endpoint),
    None,
}

pub fn scan() -> CmdInput {
    let matches = App::new("Realm")
        .about("A high efficiency relay tool")
        .version(format!("{} {}", VERSION, FEATURES).as_str())
        .license("MIT")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableVersionFlag)
        .setting(
            AppSettings::DisableHelpFlag | AppSettings::DisableHelpSubcommand,
        )
        .override_usage("realm [FLAGS] [OPTIONS]")
        .help_heading("FLAGS")
        .args(&[
            Arg::new("udp")
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
        .help_heading("OPTIONS")
        .args(&[
            Arg::new("config")
                .short('c')
                .long("config")
                .about("use config file")
                .value_name("path")
                .takes_value(true)
                .display_order(0),
            Arg::new("local")
                .short('l')
                .long("listen")
                .about("listen address")
                .value_name("addr")
                .takes_value(true)
                .display_order(1),
            Arg::new("remote")
                .short('r')
                .long("remote")
                .about("remote address")
                .value_name("addr")
                .takes_value(true)
                .display_order(2),
            Arg::new("through")
                .short('x')
                .long("through")
                .about("send through ip or address")
                .value_name("addr")
                .takes_value(true)
                .display_order(3),
            Arg::new("tcp_timeout")
                .long("tcp-timeout")
                .about("set timeout value for tcp")
                .value_name("second")
                .takes_value(true)
                .display_order(4),
            Arg::new("udp_timeout")
                .long("udp-timeout")
                .about("set timeout value for udp")
                .value_name("second")
                .takes_value(true)
                .display_order(5),
        ])
        .get_matches();

    parse_matches(matches)
}

fn parse_matches(matches: ArgMatches) -> CmdInput {
    #[cfg(unix)]
    if matches.is_present("daemon") {
        crate::utils::daemonize();
    }

    if let Some(config) = matches.value_of("config") {
        return CmdInput::Config(config.to_string());
    }

    if let (Some(local), Some(remote)) =
        (matches.value_of("local"), matches.value_of("remote"))
    {
        let tcp_timeout = matches
            .value_of("tcp_timeout")
            .map_or(TCP_TIMEOUT, |t| t.parse::<usize>().unwrap_or(TCP_TIMEOUT));
        let udp_timeout = matches
            .value_of("udp_timeout")
            .map_or(UDP_TIMEOUT, |t| t.parse::<usize>().unwrap_or(UDP_TIMEOUT));
        return CmdInput::Endpoint(Endpoint::new(
            local,
            remote,
            matches.value_of("through").unwrap_or(""),
            matches.is_present("udp"),
            matches.is_present("fast_open"),
            matches.is_present("zero_copy"),
            tcp_timeout,
            udp_timeout,
        ));
    }

    CmdInput::None
}
