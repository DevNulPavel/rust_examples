use std::{fmt::Display, path::PathBuf};
use structopt::StructOpt;

/// App parameters
#[derive(Debug)]
pub enum CompressionArg {
    None,
    Brotli,
    Gzip,
}
impl Display for CompressionArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionArg::Brotli => {
                write!(f, "brotli")
            }
            CompressionArg::Gzip => {
                write!(f, "gzip")
            }
            CompressionArg::None => {
                write!(f, "none")
            }
        }
    }
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
    pub compression_type: CompressionArg,

    /// Compression level if compression using enabled, value from 1 to 11
    #[structopt(
        long,
        required_if("compression_type", "CompressionArg::Gzip"),
        required_if("compression_type", "CompressionArg::Brotli"),
        default_value = "0"
    )]
    pub compression_level: u8,

    #[structopt(
        long,
        parse(from_os_str),
        required_if("compression_type", "CompressionArg::Gzip"),
        required_if("compression_type", "CompressionArg::Brotli")
    )]
    pub compression_cache_path: Option<PathBuf>,

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
