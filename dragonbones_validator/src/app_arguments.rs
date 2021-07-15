use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Source directory with dragonbones for validation
    #[structopt(long, parse(from_os_str))]
    pub source_directory: PathBuf,

    /// Multiply X2 texture size
    #[structopt(long)]
    pub x2_texture_size: bool,

    /// Verbose level
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8
}
