use super::{global::get_global_reactor, this::Reactor};
use mio::{event::Source, Interest, Token};
use std::{
    io::{ErrorKind, Read, Result, Write},
    pin::Pin,
    task::{Context, Poll, Waker},
};

////////////////////////////////////////////////////////////////////////////////

pub(super) struct IoHandle<S>
where
    S: Source,
{
    /// Непосредственно сокет, который мы хотели бы слушать
    source: S,

    /// Ждем чтения?
    waiting_read: bool,

    /// Ждем записи?
    waiting_write: bool,

    /// Токен непосредственно в slab + зарегистрированный для пробуждения в mio
    token: Option<Token>,
}

// TODO: Но вроде бы это и так у нас будет автоматически?
/// Явно помечаем дополнительно, что у нас IoHandle не является запинированным если S: Source.
/// Видимо, это нужно чтобы указать Unpin только для определенных типов S.
impl<S> Unpin for IoHandle<S> where S: Source {}

impl<S> IoHandle<S>
where
    S: Source,
{
    /// Создаем новый
    pub(super) fn new(source: S) -> Self {
        Self {
            source,
            waiting_read: false,
            waiting_write: false,
            token: None,
        }
    }

    /// Ссылка на Source
    pub(super) fn source(&self) -> &S {
        &self.source
    }

    /// Регистрируем данный сокет для отслеживания событий
    pub(super) fn register(&mut self, interest: Interest, waker: Waker) -> Result<()> {
        // Есть ли уже токен регистрации
        match self.token {
            // Токен есть
            Some(token) => {
                // Заново регистрируем данный сокет, удаляя старую регистрацию
                get_global_reactor().reregister(token, &mut self.source, interest, waker)?;
            }
            None => {
                // Нету текущего токена, так что регистрируем работу
                let token = get_global_reactor().register(&mut self.source, interest, waker)?;

                // Сохраним этот токен
                self.token = Some(token);
            }
        };

        Ok(())
    }

    /// Снимаем регистрацию
    pub(super) fn deregister(&mut self) -> Result<()> {
        match self.token {
            Some(token) => {
                // Снимаем регистрацию для текущего токена и сокета
                get_global_reactor().deregister(token, &mut self.source)?;

                // Сброс
                self.token = None;

                Ok(())
            }
            None => Ok(()),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

CONTINUE HERE
impl<S> IoHandle<S>
where
    S: Source + Read,
{
    pub(super) fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        if !self.waiting_read {
            self.register(Interest::READABLE, cx.waker().clone())?;
            self.waiting_read = true;
            return Poll::Pending;
        }

        match self.source.read(buf) {
            Ok(n) => {
                if n == 0 {
                    self.deregister()?;
                    self.waiting_read = false;
                }

                Poll::Ready(Ok(n))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<S> IoHandle<S>
where
    S: Source + Write,
{
    pub(super) fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        if !self.waiting_write {
            self.register(Interest::WRITABLE, cx.waker().clone())?;
            self.waiting_write = true;
            return Poll::Pending;
        }

        match self.source.write(buf) {
            Ok(n) => {
                if n == 0 {
                    self.deregister()?;
                    self.waiting_write = false;
                }

                Poll::Ready(Ok(n))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }

    pub(super) fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        if !self.waiting_write {
            self.register(Interest::WRITABLE, cx.waker().clone())?;
            self.waiting_write = true;
            return Poll::Pending;
        }

        match self.source.flush() {
            Ok(()) => {
                self.deregister()?;
                self.waiting_read = false;
                self.waiting_write = false;
                Poll::Ready(Ok(()))
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl<S> Drop for IoHandle<S>
where
    S: Source,
{
    fn drop(&mut self) {
        self.deregister().unwrap()
    }
}
