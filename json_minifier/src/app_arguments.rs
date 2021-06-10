use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Input folder
    #[structopt(long, parse(from_os_str))]
    pub input_folder: PathBuf,
    
    /// Verbose
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
