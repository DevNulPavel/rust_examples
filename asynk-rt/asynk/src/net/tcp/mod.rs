pub(crate) mod stream;

use super::TcpStream;
use crate::reactor::io_handle::IoHandle;
use futures::Stream;
use mio::{net::TcpListener as MioTcpListener, Interest};
use std::{
    io::{self, Result},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

pub struct TcpListener(MioTcpListener);

impl TcpListener {
    pub fn bind(addr: SocketAddr) -> Result<Self> {
        Ok(Self(MioTcpListener::bind(addr)?))
    }

    pub fn accept(self) -> Accept {
        self.0.into()
    }
}

pub struct Accept(IoHandle<MioTcpListener>);

impl From<MioTcpListener> for Accept {
    fn from(source: MioTcpListener) -> Self {
        Self(IoHandle::new(source))
    }
}

impl Stream for Accept {
    type Item = Result<(TcpStream, SocketAddr)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.0.source().accept() {
            Ok((stream, addr)) => Poll::Ready(Some(Ok((stream.into(), addr)))),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.0.register(Interest::READABLE, cx.waker().clone())?;
                Poll::Pending
            }
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}
