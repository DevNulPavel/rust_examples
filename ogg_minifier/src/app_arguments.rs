use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct TargetParams {
    /// Max bitrate
    #[structopt(long)]
    pub max_bitrate: u32,

    /// Max freq
    #[structopt(long)]
    pub max_freq: u32,
}

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Ogg files directory
    #[structopt(long, parse(from_os_str))]
    pub ogg_files_directory: PathBuf,

    /// Cache path
    // #[structopt(long, parse(from_os_str))]
    // pub cache_path: PathBuf,

    // TODO: Flatten params
    #[structopt(flatten)]
    pub target_params: TargetParams,

    /// Verbose
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8
}
