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
        CmdInput::None => {}
    }
}

fn start_from_config(c: String) {
    let config = GlobalConfig::from_config_file(&c);
    relay::init_resolver(config.dns_mode.to_strategy());
    let eps: Vec<Endpoint> = config
        .endpoints
        .into_iter()
        .map(|epc| epc.to_endpoint())
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
