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
        .version("1.3-custom")
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
            Arg::with_name("udp")
                .short("u")
                .long("udp")
                .help("enable udp"),
        )
        .subcommand(
            SubCommand::with_name("nav")
                .about("An Interactive configuration editor")
                .version("0.1.0")
                .author("zephyr <i@zephyr.moe>"),
        )
        .get_matches();

    if let Some(config) = matches.value_of("config") {
        return CmdInput::Config(config.to_string());
    }

    if let (Some(local), Some(remote)) =
        (matches.value_of("local"), matches.value_of("remote"))
    {
        return CmdInput::Endpoint(Endpoint::new(
            local,
            remote,
            matches.is_present("udp"),
        ));
    }

    if matches.subcommand_matches("nav").is_some() {
        return CmdInput::Navigate;
    }

    CmdInput::None
}
