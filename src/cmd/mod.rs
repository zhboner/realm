use clap::{Arg, App, ArgMatches, AppSettings};

use super::Endpoint;
use crate::utils::TCP_TIMEOUT;
use crate::utils::UDP_TIMEOUT;

pub enum CmdInput {
    Config(String),
    Endpoint(Endpoint),
    None,
}

pub fn scan() -> CmdInput {
    let matches = App::new("Realm")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(super::VERSION)
        .about("A high efficiency proxy tool")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("use config file")
                .value_name("path")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("local")
                .short("l")
                .long("listen")
                .help("listen address")
                .value_name("addr")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("remote")
                .short("r")
                .long("remote")
                .help("remote address")
                .value_name("addr")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("tcp_timeout")
                .long("tcp-timeout")
                .help("set timeout value for tcp")
                .value_name("second")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("udp_timeout")
                .long("udp-timeout")
                .help("set timeout value for udp")
                .value_name("second")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("through")
                .short("x")
                .long("through")
                .help("send through ip or address")
                .value_name("addr")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("udp")
                .short("u")
                .long("udp")
                .help("enable udp"),
        )
        .arg(
            Arg::with_name("fast_open")
                .short("f")
                .long("tfo")
                .help("enable tfo"),
        )
        .arg(
            Arg::with_name("zero_copy")
                .short("z")
                .long("zero-copy")
                .help("enable tcp zero-copy"),
        )
        .arg(
            Arg::with_name("daemon")
                .short("d")
                .long("daemon")
                .help("daemonize"),
        )
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
