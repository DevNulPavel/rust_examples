use super::{accept::AcceptFuture, TcpStream};
use crate::reactor::IoHandle;
use futures::Stream;
use mio::{net::TcpListener as MioTcpListener, Interest};
use std::{
    io::{self, Result},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

////////////////////////////////////////////////////////////////////////////////

pub struct TcpListener(MioTcpListener);

impl TcpListener {
    pub fn bind(addr: SocketAddr) -> Result<Self> {
        Ok(Self(MioTcpListener::bind(addr)?))
    }

    pub fn accept(self) -> AcceptFuture {
        self.0.into()
    }
}
