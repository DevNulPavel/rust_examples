use std::{
    io::{
        ErrorKind
    }
};

#[derive(Debug)]
pub enum UtpError {
    Malformed,
    UnknownPacketType,
    WrongVersion,
    FamillyMismatch,
    PacketLost,
    MustClose,
    IO(std::io::Error),
    RecvError(async_std::sync::RecvError),
}

impl UtpError {
    pub fn should_continue(&self) -> bool {
        match self {
            UtpError::IO(ref e) if e.kind() == ErrorKind::TimedOut
                || e.kind() == ErrorKind::WouldBlock => {
                true
            }
            _ => false
        }
    }
}

impl From<std::io::Error> for UtpError {
    fn from(e: std::io::Error) -> UtpError {
        UtpError::IO(e)
    }
}