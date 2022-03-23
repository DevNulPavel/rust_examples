
use snafu::Snafu;
use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum MerinoError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Socks error: {0}")]
    Socks(#[from] ResponseCode),
}

#[derive(Debug, Snafu)]
/// Possible SOCKS5 Response Codes
pub enum ResponseCode {
    Success = 0x00,
    #[snafu(display("SOCKS5 Server Failure"))]
    Failure = 0x01,
    #[snafu(display("SOCKS5 Rule failure"))]
    RuleFailure = 0x02,
    #[snafu(display("network unreachable"))]
    NetworkUnreachable = 0x03,
    #[snafu(display("host unreachable"))]
    HostUnreachable = 0x04,
    #[snafu(display("connection refused"))]
    ConnectionRefused = 0x05,
    #[snafu(display("TTL expired"))]
    TtlExpired = 0x06,
    #[snafu(display("Command not supported"))]
    CommandNotSupported = 0x07,
    #[snafu(display("Addr Type not supported"))]
    AddrTypeNotSupported = 0x08,
}

impl From<MerinoError> for ResponseCode {
    fn from(e: MerinoError) -> Self {
        match e {
            MerinoError::Socks(e) => e,
            MerinoError::Io(_) => ResponseCode::Failure,
        }
    }
}
