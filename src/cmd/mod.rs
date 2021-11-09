use clap::{Arg, App, SubCommand};

use super::Endpoint;

mod nav;
pub use nav::run_navigator;

pub enum CmdInput {
    Config(String),
    Endpoint(Endpoint),
    Navigate,
    None,
}

pub fn scan() -> CmdInput {
    let matches = App::new("Realm")
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
        .subcommand(
            SubCommand::with_name("nav")
                .about("An Interactive configuration editor")
                .version("0.1.0")
                .author("zephyr <i@zephyr.moe>"),
        )
        .get_matches();

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
        return CmdInput::Endpoint(Endpoint::new(
            local,
            remote,
            matches.value_of("through").unwrap_or(""),
            matches.is_present("udp"),
            matches.is_present("fast_open"),
            matches.is_present("zero_copy"),
        ));
    }

    if matches.subcommand_matches("nav").is_some() {
        return CmdInput::Navigate;
    }

    CmdInput::None
}
