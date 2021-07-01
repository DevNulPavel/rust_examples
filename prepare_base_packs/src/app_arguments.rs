use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Config json
    #[structopt(long, parse(from_os_str))]
    pub config_json: PathBuf,

    /// Source directories with res folder
    #[structopt(long, parse(from_os_str))]
    pub source_directories: Vec<PathBuf>,

    /// Pack directories with res subfolders
    #[structopt(long, parse(from_os_str))]
    pub packs_directory: PathBuf,

    /// Pack directory prefixes
    #[structopt(long)]
    pub packs_directory_prefixes: Vec<String>,

    /// Target client res directory
    #[structopt(long, parse(from_os_str))]
    pub target_client_res_directory: PathBuf,

    /// Target server res directory
    #[structopt(long, parse(from_os_str))]
    pub target_server_res_directory: PathBuf,

    /// Verbose
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8
}
