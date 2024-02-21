use std::fmt::{Formatter, Display};
use serde::{Serialize, Deserialize};
use log::LevelFilter;
use super::Config;
use crate::consts::DEFAULT_LOG_FILE;

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    #[default]
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<String> for LogLevel {
    fn from(x: String) -> Self {
        use LogLevel::*;
        match x.to_ascii_lowercase().as_str() {
            "off" => Off,
            "error" => Error,
            "warn" => Warn,
            "info" => Info,
            "debug" => Debug,
            "trace" => Trace,
            _ => Self::default(),
        }
    }
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

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use LogLevel::*;
        let s = match self {
            Off => "off",
            Error => "error",
            Warn => "warn",
            Info => "info",
            Debug => "debug",
            Trace => "trace",
        };
        write!(f, "{}", s)
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct LogConf {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<LogLevel>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

impl Config for LogConf {
    type Output = (LevelFilter, fern::Output);

    fn is_empty(&self) -> bool {
        crate::empty![self => level, output]
    }

    fn build(self) -> Self::Output {
        use std::io;
        use std::fs::OpenOptions;
        let LogConf { level, output } = self;
        let level = level.unwrap_or_default();
        let output = output.unwrap_or_else(|| String::from(DEFAULT_LOG_FILE));

        let output: fern::Output = match output.as_str() {
            "stdout" => io::stdout().into(),
            "stderr" => io::stderr().into(),
            output => OpenOptions::new()
                .append(true)
                .create(true)
                .open(output)
                .unwrap_or_else(|e| panic!("failed to open {}: {}", output, &e))
                .into(),
        };

        (level.into(), output)
    }

    fn rst_field(&mut self, other: &Self) -> &mut Self {
        use crate::rst;
        let other = other.clone();

        rst!(self, level, other);
        rst!(self, output, other);
        self
    }

    fn take_field(&mut self, other: &Self) -> &mut Self {
        use crate::take;
        let other = other.clone();

        take!(self, level, other);
        take!(self, output, other);
        self
    }

    fn from_cmd_args(matches: &clap::ArgMatches) -> Self {
        let level = matches.get_one::<String>("log_level").cloned().map(LogLevel::from);

        let output = matches.get_one("log_output").cloned();

        Self { level, output }
    }
}

impl Display for LogConf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let LogConf { level, output } = self.clone();
        let level = level.unwrap_or_default();
        let output = output.unwrap_or_else(|| String::from("stdout"));

        write!(f, "level={}, output={}", level, output)
    }
}
