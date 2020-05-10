use structopt::StructOpt;

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

pub fn parse_arguments() -> Cli {
    return Cli::from_args();
}
