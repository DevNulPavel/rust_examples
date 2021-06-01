use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Config path
    #[structopt(long, parse(from_os_str))]
    pub dynamic_packs_config_path: PathBuf,

    /// Resources directory
    #[structopt(long, parse(from_os_str))]
    pub resources_directory: PathBuf,

    /// Output resources config path
    #[structopt(long, parse(from_os_str))]
    pub output_resources_config_path: PathBuf,

    /// Output dynamic packs dir
    #[structopt(long, parse(from_os_str))]
    pub output_dynamic_packs_dir: PathBuf,

    /// Max pack size
    #[structopt(long, default_value = "512000")] // 500 * 1024 = 512000
    pub max_pack_size: u64,

    /// Verbose
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8
}
