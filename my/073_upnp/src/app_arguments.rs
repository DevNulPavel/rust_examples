use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Verbose
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
