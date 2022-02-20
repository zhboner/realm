use clap::{Command, Arg, ArgMatches};

pub fn add_all(app: Command) -> Command {
    let app = add_convert(app);
    app
}

pub fn add_convert(app: Command) -> Command {
    let cvt = Command::new("convert")
        .version("0.0.1")
        .about("Convert your legacy configuration into an advanced one");

    let app = app.subcommand(cvt);
    app
}
