#![allow(clippy::let_and_return)]
use clap::{Command, Arg, ArgAction};

pub fn add_all(app: Command) -> Command {
    let app = add_flags(app);
    let app = add_options(app);
    let app = add_global_options(app);
    app
}

pub fn add_flags(app: Command) -> Command {
    app.next_help_heading("FLAGS").args(&[
        Arg::new("help")
            .short('h')
            .long("help")
            .help("show help")
            .action(ArgAction::SetTrue)
            .display_order(0),
        Arg::new("version")
            .short('v')
            .long("version")
            .help("show version")
            .action(ArgAction::SetTrue)
            .display_order(1),
        Arg::new("daemon")
            .short('d')
            .long("daemon")
            .help("run as a unix daemon")
            .action(ArgAction::SetTrue)
            .display_order(2),
        Arg::new("use_udp")
            .short('u')
            .long("udp")
            .help("force enable udp forward")
            .action(ArgAction::SetTrue)
            .display_order(3),
        Arg::new("no_tcp")
            .short('t')
            .long("ntcp")
            .help("force disable tcp forward")
            .action(ArgAction::SetTrue)
            .display_order(4),
        Arg::new("ipv6_only")
            .short('6')
            .long("ipv6")
            .help("force disable ipv6 mapped ipv4")
            .action(ArgAction::SetTrue)
            .display_order(5),
        Arg::new("fast_open")
            .short('f')
            .long("tfo")
            .help("force enable tcp fast open -- deprecated")
            .action(ArgAction::SetTrue)
            .display_order(6),
        Arg::new("zero_copy")
            .short('z')
            .long("splice")
            .help("force enable tcp zero copy -- deprecated")
            .action(ArgAction::SetTrue)
            .display_order(7),
    ])
}

pub fn add_options(app: Command) -> Command {
    app.next_help_heading("OPTIONS").args(&[
        Arg::new("config")
            .short('c')
            .long("config")
            .help("use config file")
            .value_name("path")
            .display_order(0),
        Arg::new("local")
            .short('l')
            .long("listen")
            .help("listen address")
            .value_name("address")
            .display_order(1),
        Arg::new("remote")
            .short('r')
            .long("remote")
            .help("remote address")
            .value_name("address")
            .display_order(2),
        Arg::new("through")
            .short('x')
            .long("through")
            .help("send through ip or address")
            .value_name("address")
            .display_order(3),
        Arg::new("interface")
            .short('i')
            .long("interface")
            .help("send through interface")
            .value_name("device")
            .display_order(4),
        Arg::new("listen_interface")
            .short('e')
            .long("listen-interface")
            .help("listen interface")
            .value_name("device")
            .display_order(5),
        Arg::new("listen_transport")
            .short('a')
            .long("listen-transport")
            .help("listen transport")
            .value_name("options")
            .display_order(6),
        Arg::new("remote_transport")
            .short('b')
            .long("remote-transport")
            .help("remote transport")
            .value_name("options")
            .display_order(7),
    ])
}

pub fn add_global_options(app: Command) -> Command {
    // sys
    let app = app.next_help_heading("SYS OPTIONS").args(&[
        Arg::new("nofile")
            .short('n')
            .long("nofile")
            .help("set nofile limit")
            .value_name("limit")
            .display_order(0),
        Arg::new("pipe_page")
            .short('p')
            .long("pipe-page")
            .help("set pipe capacity")
            .value_name("number")
            .display_order(1),
        Arg::new("pre_conn_hook")
            .short('j')
            .long("pre-conn-hook")
            .help("set pre-connect hook")
            .value_name("path")
            .display_order(2),
    ]);

    // log
    let app = app.next_help_heading("LOG OPTIONS").args(&[
        Arg::new("log_level")
            .long("log-level")
            .help("override log level")
            .value_name("level")
            .display_order(0),
        Arg::new("log_output")
            .long("log-output")
            .help("override log output")
            .value_name("path")
            .display_order(1),
    ]);

    // dns
    let app = app.next_help_heading("DNS OPTIONS").args(&[
        Arg::new("dns_mode")
            .long("dns-mode")
            .help("override dns mode")
            .value_name("mode")
            .display_order(0),
        Arg::new("dns_min_ttl")
            .long("dns-min-ttl")
            .help("override dns min ttl")
            .value_name("second")
            .display_order(1),
        Arg::new("dns_max_ttl")
            .long("dns-max-ttl")
            .help("override dns max ttl")
            .value_name("second")
            .display_order(2),
        Arg::new("dns_cache_size")
            .long("dns-cache-size")
            .help("override dns cache size")
            .value_name("number")
            .display_order(3),
        Arg::new("dns_protocol")
            .long("dns-protocol")
            .help("override dns protocol")
            .value_name("protocol")
            .display_order(4),
        Arg::new("dns_servers")
            .long("dns-servers")
            .help("override dns servers")
            .value_name("servers")
            .display_order(5),
    ]);

    // proxy-protocol belogs to network
    let app = app.next_help_heading("PROXY OPTIONS").args([
        Arg::new("send_proxy")
            .long("send-proxy")
            .help("send proxy protocol header")
            .display_order(0),
        Arg::new("send_proxy_version")
            .long("send-proxy-version")
            .help("send proxy protocol version")
            .value_name("version")
            .display_order(1),
        Arg::new("accept_proxy")
            .long("accept-proxy")
            .help("accept proxy protocol header")
            .display_order(2),
        Arg::new("accept_proxy_timeout")
            .long("accept-proxy-timeout")
            .help("accept proxy protocol timeout")
            .value_name("second")
            .display_order(3),
    ]);

    // timeout belogs to network
    let app = app.next_help_heading("TIMEOUT OPTIONS").args([
        Arg::new("tcp_timeout")
            .long("tcp-timeout")
            .help("override tcp timeout(5s)")
            .value_name("second")
            .display_order(0),
        Arg::new("udp_timeout")
            .long("udp-timeout")
            .help("override udp timeout(30s)")
            .value_name("second")
            .display_order(1),
        Arg::new("tcp_keepalive")
            .long("tcp-keepalive")
            .help("override default tcp keepalive interval(15s)")
            .value_name("second")
            .display_order(2),
        Arg::new("tcp_keepalive_probe")
            .long("tcp-keepalive-probe")
            .help("override default tcp keepalive count(3)")
            .value_name("count")
            .display_order(3),
    ]);

    app
}
