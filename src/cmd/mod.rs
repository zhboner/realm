use clap::{Arg, App, SubCommand};

mod nav;
pub use nav::run_navigator;

pub enum CmdInput {
    Config(String),
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
                .value_name("json config file")
                .help("specify a config file in json format")
                .takes_value(true),
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
    if let Some(_) = matches.subcommand_matches("nav") {
        return CmdInput::Navigate;
    }
    CmdInput::None
}
