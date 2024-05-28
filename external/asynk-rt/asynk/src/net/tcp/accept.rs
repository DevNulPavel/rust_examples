use super::TcpStream;
use crate::reactor::{IoHandle, IoHandleOwned, IoHandleRef};
use futures::Stream;
use mio::{net::TcpListener as MioTcpListener, Interest};
use std::{
    io::{self, Result},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

////////////////////////////////////////////////////////////////////////////////

/// Футура ожидания нового подключения
pub struct AcceptFuture<'a> {
    pub(super) handle: IoHandleRef<'a, MioTcpListener>,
}

// /// Поддержка создания из листнера
// impl<'a> From<MioTcpListener> for AcceptFuture {
//     fn from(source: MioTcpListener) -> Self {
//         Self(IoHandle::new(source))
//     }
// }

impl<'a> Stream for AcceptFuture<'a> {
    /// Результатом нового подключения будет являться новый стрим и адрес
    type Item = Result<(TcpStream, SocketAddr)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.handle.source().accept() {
            Ok((stream, addr)) => Poll::Ready(Some(Ok((stream.into(), addr)))),

            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                self.handle
                    .register(Interest::READABLE, cx.waker().clone())?;

                Poll::Pending
            }
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}
