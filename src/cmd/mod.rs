use clap::{Command, ArgMatches};

use realm_core::realm_io;
use realm_core::realm_syscall;

use crate::conf::CmdOverride;
use crate::conf::EndpointConf;
use crate::conf::{Config, LogConf, DnsConf, NetConf};

use crate::VERSION;
use crate::consts::FEATURES;

mod sub;
mod flag;

#[allow(clippy::large_enum_variant)]
pub enum CmdInput {
    Config(String, CmdOverride),
    Endpoint(EndpointConf, CmdOverride),
    None,
}

pub fn scan() -> CmdInput {
    let ver = format!("{} {}", VERSION, FEATURES);
    let app = Command::new("Realm").about("A high efficiency relay tool").version(ver);

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

    if matches.get_flag("help") {
        app2.print_help().unwrap();
        return CmdInput::None;
    }

    if matches.get_flag("version") {
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
    if matches.get_flag("daemon") {
        realm_syscall::daemonize("realm is running in the background");
    }

    #[cfg(all(unix, not(target_os = "android")))]
    {
        use realm_syscall::get_nofile_limit;
        use realm_syscall::set_nofile_limit;
        use realm_syscall::bump_nofile_limit;

        // set
        if let Some(nofile) = matches.get_one::<String>("nofile") {
            if let Ok(nofile) = nofile.parse::<u64>() {
                let _ = set_nofile_limit(nofile);
            } else {
                eprintln!("invalid nofile value: {}", nofile);
            }
        } else {
            let _ = bump_nofile_limit();
        }

        // get
        if let Ok((soft, hard)) = get_nofile_limit() {
            println!("fd: soft={}, hard={}", soft, hard);
        }
    }

    #[cfg(target_os = "linux")]
    {
        use realm_io::set_pipe_size;

        if let Some(page) = matches.get_one::<String>("pipe_page") {
            if let Ok(page) = page.parse::<usize>() {
                set_pipe_size(page * 0x1000);
                println!("pipe capacity: {}", page * 0x1000);
            } else {
                eprintln!("invalid page value: {}", page);
            }
        }
    }

    #[cfg(feature = "hook")]
    {
        use realm_core::hook::pre_conn::load_dylib as load_pre_conn;
        if let Some(path) = matches.get_one::<String>("pre_conn_hook") {
            load_pre_conn(path);
            println!("hook: {}", path);
        }
    }

    let opts = parse_global_opts(&matches);

    if let Some(config) = matches.get_one("config").cloned() {
        return CmdInput::Config(config, opts);
    }

    if matches.contains_id("local") && matches.contains_id("remote") {
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
