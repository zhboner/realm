use std::thread;
// use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::net::{TcpListener, TcpStream, Shutdown};
use structopt::StructOpt;

mod relay;



/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Cli {
    /// The pattern to look for
    #[structopt(short = "l", long = "local")]
    client: String,
    /// The path to the file to read
    #[structopt(short = "r", long = "remote")]
    remote: String,
}



fn main() {
    let cli = Cli::from_args();
    let client_socket: SocketAddr = cli.client.parse().expect("Unable to parse client address");
    let remote_socket: SocketAddr = cli.remote.parse().expect("Unable to parse remote address");

    relay::start(client_socket, remote_socket);
}
