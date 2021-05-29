use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Input directory
    #[structopt(long, parse(from_os_str))]
    pub input_directory: PathBuf,

    /// Ignore folders list
    #[structopt(long, parse(from_os_str))]
    pub ignore_directories: Vec<PathBuf>,

    /// Output file path
    #[structopt(long, parse(from_os_str))]
    pub output_file: PathBuf
}
