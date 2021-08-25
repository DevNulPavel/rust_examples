use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Language files folder
    #[structopt(long, parse(from_os_str))]
    pub lang_files_folder: PathBuf,
    
    /// Filter config json file path
    #[structopt(long, parse(from_os_str))]
    pub filter_config_path: PathBuf,

    /// Verbose
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
