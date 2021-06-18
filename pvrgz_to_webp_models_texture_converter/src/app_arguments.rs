use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Pvrgz model texture directories
    #[structopt(long, parse(from_os_str))]
    pub pvrgz_directories: Vec<PathBuf>,

    /// Models config json
    #[structopt(long, parse(from_os_str))]
    pub models_config_json: PathBuf,

    /// Cache path
    #[structopt(long, parse(from_os_str))]
    pub cache_path: PathBuf,

    /// Ignore json array file
    #[structopt(long, parse(from_os_str))]
    pub ignore_config_path: Option<PathBuf>,

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
