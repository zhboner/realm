mod cmd;
mod relay;
mod utils;
mod config;

use cmd::CmdInput;
use config::GlobalConfig;
use relay::Endpoint;

fn main() {
    match cmd::scan() {
        CmdInput::Config(c) => start_from_config(c),
        CmdInput::Endpoint(ep) => start_from_cmd(ep),
        CmdInput::Navigate => cmd::run_navigator(),
        CmdInput::None => {}
    }
}

fn start_from_cmd(c: Endpoint) {
    run_relay(vec![c])
}

fn start_from_config(c: String) {
    let config = GlobalConfig::from_config_file(&c);
    relay::init_resolver(config.dns_mode.into());
    let eps: Vec<Endpoint> = config
        .endpoints
        .into_iter()
        .map(|epc| epc.into_endpoint())
        .collect();
    run_relay(eps);
}

fn run_relay(eps: Vec<Endpoint>) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(relay::run(eps))
}
