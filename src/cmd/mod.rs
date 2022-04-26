use clap::{Command, ArgMatches};

use crate::conf::CmdOverride;
use crate::conf::EndpointConf;
use crate::conf::{Config, LogConf, DnsConf, NetConf};

use crate::VERSION;
use crate::utils::FEATURES;

mod sub;
mod flag;

pub enum CmdInput {
    Config(String, CmdOverride),
    Endpoint(EndpointConf, CmdOverride),
    None,
}

pub fn scan() -> CmdInput {
    let version = format!("{} {}", VERSION, FEATURES);
    let app = Command::new("Realm")
        .about("A high efficiency relay tool")
        .version(version.as_str());

    let app = app
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg_required_else_help(true)
        .override_usage("realm [FLAGS] [OPTIONS]");

    let app = flag::add_all(app);
    let app = sub::add_all(app);

    // do other things
    let mut app2 = app.clone();
    let matches = app.get_matches();

    if matches.is_present("help") {
        app2.print_help().unwrap();
        return CmdInput::None;
    }

    if matches.is_present("version") {
        print!("{}", app2.render_version());
        return CmdInput::None;
    }

    #[allow(clippy::single_match)]
    match matches.subcommand() {
        Some(("convert", sub_matches)) => {
            sub::handle_convert(sub_matches);
            return CmdInput::None;
        }
        _ => {}
    };

    // start
    handle_matches(matches)
}

fn handle_matches(matches: ArgMatches) -> CmdInput {
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

    #[cfg(all(target_os = "linux", feature = "zero-copy"))]
    {
        use realm_io::set_pipe_size;

        if let Some(page) = matches.value_of("pipe_page") {
            if let Ok(page) = page.parse::<usize>() {
                set_pipe_size(page * 0x1000);
                println!("pipe capacity: {}", page * 0x1000);
            }
        }
    }

    let opts = parse_global_opts(&matches);

    if let Some(config) = matches.value_of("config") {
        return CmdInput::Config(String::from(config), opts);
    }

    if matches.value_of("local").is_some() && matches.value_of("remote").is_some() {
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
