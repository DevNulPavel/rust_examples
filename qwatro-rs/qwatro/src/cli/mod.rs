pub mod port_scan;
pub mod proxy;

use crate::cli::port_scan::PortScanArgs;
use crate::cli::proxy::ProxyArgs;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Port scan
    PS(PortScanArgs),
    /// Proxy
    #[command(subcommand)]
    Proxy(ProxyArgs),
}
