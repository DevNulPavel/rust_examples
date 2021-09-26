use std::path::PathBuf;
use structopt::StructOpt;

/// App parameters
#[derive(Debug)]
pub enum CompressionArg {
    None,
    Brotli,
    Gzip,
}

fn parse_compression(src: &str) -> CompressionArg {
    match src {
        "gzip" => CompressionArg::Gzip,
        "brotli" => CompressionArg::Brotli,
        _ => CompressionArg::None,
    }
}

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Result dirs count
    #[structopt(long)]
    pub resuld_dirs_count: usize,

    /// Result dirs path
    #[structopt(long, parse(from_os_str))]
    pub result_dirs_path: PathBuf,

    /// Use compression before size analyzing, supported values: gzip, brotli
    #[structopt(long, parse(from_str = parse_compression))]
    pub use_compression: CompressionArg,

    /// Compression level if compression using enabled, value from 1 to 11
    #[structopt(
        long,
        required_if("use_compression", "CompressionArg::Gzip"),
        required_if("use_compression", "CompressionArg::Brotli"),
        default_value = "0"
    )]
    pub compression_level: u8,

    /// Source directories root
    #[structopt(long, parse(from_os_str))]
    pub source_dirs_root: PathBuf,

    /// Source directories
    #[structopt(long, parse(from_os_str))]
    pub source_dirs: Vec<PathBuf>,

    /// Verbose
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
