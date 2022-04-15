use clap::{Command, Arg};

#[allow(clippy::let_and_return)]
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
            .display_order(0),
        Arg::new("version")
            .short('v')
            .long("version")
            .help("show version")
            .display_order(1),
        Arg::new("daemon")
            .short('d')
            .long("daemon")
            .help("run as a unix daemon")
            .display_order(2),
        Arg::new("use_udp")
            .short('u')
            .long("udp")
            .help("force enable udp forward")
            .display_order(3),
        Arg::new("fast_open")
            .short('f')
            .long("tfo")
            .help("force enable tcp fast open")
            .display_order(4),
        Arg::new("zero_copy")
            .short('z')
            .long("splice")
            .help("force enable tcp zero copy")
            .display_order(5),
    ])
}

pub fn add_options(app: Command) -> Command {
    app.next_help_heading("OPTIONS").args(&[
        Arg::new("nofile")
            .short('n')
            .long("nofile")
            .help("set nofile limit")
            .value_name("limit")
            .takes_value(true)
            .display_order(0),
        Arg::new("pipe_page")
            .short('p')
            .long("page")
            .help("set pipe capacity")
            .value_name("number")
            .takes_value(true)
            .display_order(1),
        Arg::new("config")
            .short('c')
            .long("config")
            .help("use config file")
            .value_name("path")
            .takes_value(true)
            .display_order(2),
        Arg::new("local")
            .short('l')
            .long("listen")
            .help("listen address")
            .value_name("address")
            .takes_value(true)
            .display_order(3),
        Arg::new("remote")
            .short('r')
            .long("remote")
            .help("remote address")
            .value_name("address")
            .takes_value(true)
            .display_order(4),
        Arg::new("through")
            .short('x')
            .long("through")
            .help("send through ip or address")
            .value_name("address")
            .takes_value(true)
            .display_order(5),
        Arg::new("interface")
            .short('i')
            .long("interface")
            .help("bind to interface")
            .value_name("device")
            .takes_value(true)
            .display_order(6),
        Arg::new("listen_transport")
            .short('a')
            .long("listen-transport")
            .help("listen transport")
            .value_name("options")
            .takes_value(true)
            .display_order(7),
        Arg::new("remote_transport")
            .short('b')
            .long("remote-transport")
            .help("remote transport")
            .value_name("options")
            .takes_value(true)
            .display_order(8),
    ])
}

pub fn add_global_options(app: Command) -> Command {
    // log
    let app = app.next_help_heading("LOG OPTIONS").args(&[
        Arg::new("log_level")
            .long("log-level")
            .help("override log level")
            .value_name("level")
            .takes_value(true)
            .display_order(0),
        Arg::new("log_output")
            .long("log-output")
            .help("override log output")
            .value_name("path")
            .takes_value(true)
            .display_order(1),
    ]);

    // dns
    let app = app.next_help_heading("DNS OPTIONS").args(&[
        Arg::new("dns_mode")
            .long("dns-mode")
            .help("override dns mode")
            .value_name("mode")
            .takes_value(true)
            .display_order(0),
        Arg::new("dns_min_ttl")
            .long("dns-min-ttl")
            .help("override dns min ttl")
            .value_name("second")
            .takes_value(true)
            .display_order(1),
        Arg::new("dns_max_ttl")
            .long("dns-max-ttl")
            .help("override dns max ttl")
            .value_name("second")
            .takes_value(true)
            .display_order(2),
        Arg::new("dns_cache_size")
            .long("dns-cache-size")
            .help("override dns cache size")
            .value_name("number")
            .takes_value(true)
            .display_order(3),
        Arg::new("dns_protocol")
            .long("dns-protocol")
            .help("override dns protocol")
            .value_name("protocol")
            .takes_value(true)
            .display_order(4),
        Arg::new("dns_servers")
            .long("dns-servers")
            .help("override dns servers")
            .value_name("servers")
            .takes_value(true)
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
            .takes_value(true)
            .display_order(1),
        Arg::new("accept_proxy")
            .long("accept-proxy")
            .help("accept proxy protocol header")
            .display_order(2),
        Arg::new("accept_proxy_timeout")
            .long("accept-proxy-timeout")
            .help("accept proxy protocol timeout")
            .value_name("second")
            .takes_value(true)
            .display_order(3),
    ]);

    // timeout belogs to network
    let app = app.next_help_heading("TIMEOUT OPTIONS").args([
        Arg::new("tcp_timeout")
            .long("tcp-timeout")
            .help("override tcp timeout")
            .value_name("second")
            .takes_value(true)
            .display_order(0),
        Arg::new("udp_timeout")
            .long("udp-timeout")
            .help("override udp timeout")
            .value_name("second")
            .takes_value(true)
            .display_order(1),
    ]);

    app
}
