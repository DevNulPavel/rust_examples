use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Source directory with dragonbones json files for validation
    #[structopt(long, parse(from_os_str))]
    pub json_files_directory: PathBuf,

    /// Alternative textures directory for dragonbones
    #[structopt(long, parse(from_os_str))]
    pub alternative_texture_files_directory: Option<PathBuf>,

    /// Multiply X2 texture size
    #[structopt(long)]
    pub x2_texture_size: bool,

    /// Verbose level
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8
}
