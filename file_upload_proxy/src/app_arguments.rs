use structopt::StructOpt;
use std::path::PathBuf;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Request token for uploading
    #[structopt(short, long, env = "UPLOADER_API_TOKEN")]
    pub uploader_api_token: String,

    /// Google Cloud Storage target bucket name
    #[structopt(short, long)]
    pub google_bucket_name: String,

    /// Google credentials file path
    #[structopt(short, long, parse(from_os_str), env = "GOOGLE_CREDENTIALS_FILE")]
    pub google_credentials_file: PathBuf,

    /// Port
    #[structopt(short, long)]
    pub port: u16,

    /// Verbose level
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
