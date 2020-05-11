use structopt::StructOpt;
use std::str::FromStr;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
pub struct Cli {
    /// The pattern to look for
    #[structopt(short = "l", long = "local")]
    pub client: String,
    /// The path to the file to read
    #[structopt(short = "r", long = "remote")]
    pub remote: String,
}

pub struct Relay_config {
    pub listening_address: String,
    pub listening_port: String,
    pub remote_address: String,
    pub remote_port: String
}

impl Default for Relay_config {
    fn default() -> Relay_config {
        Relay_config{
            listening_address: String::from("127.0.0.1"),
            listening_port: String::from("1080"),
            remote_address: String::from("127.0.0.1"),
            remote_port: String::from("8080")
        }
    }
}

pub fn parse_arguments() -> Relay_config {
    let input =  Cli::from_args();
    let client = input.client;
    let remote = input.remote;

    let client_parse: Vec<&str> = client.split(":").collect();
    if client_parse.len() != 2 {
        panic!("client address is incorrect!"); 
    }
    let listening_address = String::from_str(client_parse[0]).unwrap();

    let remote_parse: Vec<&str> = remote.split(":").collect();
    if remote_parse.len() != 2 {
        panic!("remote address is incorrect!"); 
    }

    Relay_config{
        listening_address: if listening_address == "" {String::from("127.0.0.1")} else {listening_address},
        listening_port: String::from_str(client_parse[1]).unwrap(),
        remote_address: String::from_str(remote_parse[0]).unwrap(),
        remote_port: String::from_str(remote_parse[1]).unwrap(),
    }
}

