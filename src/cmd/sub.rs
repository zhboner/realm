use std::fs;
use clap::{Command, ArgMatches};
use crate::conf::{FullConf, LegacyConf};

#[allow(clippy::let_and_return)]
pub fn add_all(app: Command) -> Command {
    let app = add_convert(app);
    app
}

pub fn add_convert(app: Command) -> Command {
    let cvt = Command::new("convert")
        .version("0.1.0")
        .about("convert your legacy configuration into an advanced one")
        .allow_missing_positional(true)
        .arg_required_else_help(true)
        .arg(clap::arg!([config]).required(true))
        .arg(
            clap::arg!(-t --type <type>)
                .required(false)
                .default_value("toml")
                .display_order(0),
        )
        .arg(clap::arg!(-o --output <path>).required(false).display_order(1));

    app.subcommand(cvt)
}

pub fn handle_convert(matches: &ArgMatches) {
    let old = matches.get_one::<String>("config").unwrap();
    let old = fs::read(old).unwrap();

    let data: LegacyConf = serde_json::from_slice(&old).unwrap();
    let data: FullConf = data.into();

    let data = match matches.get_one::<String>("type").unwrap().as_str() {
        "toml" => toml::to_string(&data).unwrap(),
        "json" => serde_json::to_string(&data).unwrap(),
        _ => unreachable!(),
    };

    if let Some(out) = matches.get_one::<String>("output") {
        fs::write(out, &data).unwrap();
    } else {
        println!("{}", &data)
    }
}
