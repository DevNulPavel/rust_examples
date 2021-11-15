use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Port
    #[structopt(short, long)]
    pub port: u16,

    /// Verbose level
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
