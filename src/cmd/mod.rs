use clap::{Arg, App, SubCommand};

pub enum CmdInput {
    Config(String),
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
        .get_matches();
    if let Some(config) = matches.value_of("config") {
        return CmdInput::Config(config.to_string());
    }
    CmdInput::None
}
