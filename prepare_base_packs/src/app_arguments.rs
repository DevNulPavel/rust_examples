use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Config json
    #[structopt(long, parse(from_os_str))]
    pub config_json: PathBuf,

    /// Pack directories
    #[structopt(long, parse(from_os_str))]
    pub packs_directory: PathBuf,

    /// Pack directory prefixes
    #[structopt(long)]
    pub packs_directory_prefixes: Vec<String>,

    /// Other source directories
    #[structopt(long, parse(from_os_str))]
    pub other_source_directories: Vec<PathBuf>,

    /// Verbose
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8
}
