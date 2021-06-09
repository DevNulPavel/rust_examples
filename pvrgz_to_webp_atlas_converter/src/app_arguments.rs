use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Source directory
    #[structopt(long, parse(from_os_str))]
    pub source_directory: PathBuf,

    /// Target directory
    #[structopt(long, parse(from_os_str))]
    pub target_directory: PathBuf,

    /// Hashes database path
    #[structopt(long, parse(from_os_str))]
    pub hashes_db_path: PathBuf,

    /// Target webp quality
    #[structopt(long)]
    pub target_webp_quality: u8,

    /// Minimum pvrgz size for convert
    #[structopt(long)]
    pub minimum_pvrgz_size: u64,

    /// Verbose
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8
}
