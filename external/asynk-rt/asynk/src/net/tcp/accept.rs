use super::TcpStream;
use crate::reactor::IoHandleRef;
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

impl<'a> Stream for AcceptFuture<'a> {
    /// Результатом нового подключения будет являться новый стрим и адрес
    type Item = Result<(TcpStream, SocketAddr)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Из указанного листнера пробуем один раз получить новое соединение
        match self.handle.source().accept() {
            // Есть новое подключение
            Ok((stream, addr)) => Poll::Ready(Some(Ok((stream.into(), addr)))),

            // Подключения нового нету никакого, операция потребут блокировки и ожидания
            Err(ref e) if (e.kind() == io::ErrorKind::WouldBlock) => {
                // Поэтому мы простре регистрируем вейкер
                // для пробуждения очередного нашей футуры
                self.handle
                    .register(Interest::READABLE, cx.waker().clone())?;

                // Дальше ждем пробуждения
                Poll::Pending
            }

            // Какая-то другая ошибка, ее вернем наверх просто
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}
