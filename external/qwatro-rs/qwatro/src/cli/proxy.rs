use clap::{Parser, Subcommand};
use std::net::{AddrParseError, SocketAddr};

#[derive(Debug, Subcommand)]
pub enum ProxyArgs {
    /// TCP
    TCP(HostToServer),
}

#[derive(Debug, Parser)]
pub struct HostToServer {
    #[arg(
        help = "List of host to server mapping. Example: 127.0.0.1:9998>127.0.0.1:9999",
        value_parser = input_parser)
    ]
    pub host_to_server: Vec<(SocketAddr, SocketAddr)>,
}

fn input_parser(s: &str) -> Result<(SocketAddr, SocketAddr), String> {
    let splitted = s.split('>').collect::<Vec<_>>();

    if splitted.len() < 2 {
        return Err("incorrect arguments for local-server pair".into());
    }

    let listen: SocketAddr = splitted[0]
        .parse()
        .map_err(|e: AddrParseError| e.to_string())?;

    let server: SocketAddr = splitted[1]
        .parse()
        .map_err(|e: AddrParseError| e.to_string())?;

    Ok((listen, server))
}
