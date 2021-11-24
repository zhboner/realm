use serde::{Serialize, Deserialize};
use log::LevelFilter;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(x: LogLevel) -> Self {
        use LogLevel::*;
        match x {
            Off => LevelFilter::Off,
            Error => LevelFilter::Error,
            Warn => LevelFilter::Warn,
            Info => LevelFilter::Info,
            Debug => LevelFilter::Debug,
            Trace => LevelFilter::Trace,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogConf {
    #[serde(default)]
    pub level: LogLevel,
    #[serde(default = "output")]
    pub output: String,
}

fn output() -> String {
    String::from("stdout")
}

impl Default for LogConf {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            output: output(),
        }
    }
}

impl From<LogConf> for (LevelFilter, fern::Output) {
    fn from(conf: LogConf) -> Self {
        use std::io;
        use std::fs::OpenOptions;
        let LogConf { level, ref output } = conf;
        let output: fern::Output = match output.as_str() {
            "stdout" => io::stdout().into(),
            "stderr" => io::stderr().into(),
            output => OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(output)
                .unwrap_or_else(|ref e| {
                    panic!("failed to open {}: {}", output, e)
                })
                .into(),
        };
        (level.into(), output)
    }
}
