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

/// Реализация чтения для сокета какого-то там если он реализует `Read`
impl<S> IoHandle<S>
where
    S: Source + Read,
{
    pub(super) fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        // Мы сейчас в режиме ожидания уже?
        if self.waiting_read {
            // Пробуем прочитать ранные раз уже проснулись
            match self.source.read(buf) {
                Ok(n) => {
                    // Данных нету - сокет закрылся?
                    if n == 0 {
                        // Снимаем регистрацию на события
                        self.deregister()?;
                        // Снимаем флаг
                        self.waiting_read = false;
                    }

                    // Работа закончилась - вернем результат
                    Poll::Ready(Ok(n))
                }
                // По идее, надо было бы заблокироваться на сокете, но у нас же
                // неблокирующий рантайм
                // For "reads", EWOULDBLOCK says "there isn't any data".
                // It's saying "if this were 'normal I/O', then I'd block".
                Err(ref e) if (e.kind() == ErrorKind::WouldBlock) => {
                    // Так что ждем
                    Poll::Pending
                }
                // Ошибка
                Err(e) => Poll::Ready(Err(e)),
            }
        } else {
            // Регистрируем на чтение данный сокет + его waker
            self.register(Interest::READABLE, cx.waker().clone())?;
            // Проставляем флаг
            self.waiting_read = true;
            // Дальшем будем ждать просто пробуждения
            return Poll::Pending;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Реализация поддержки записи данных
impl<S> IoHandle<S>
where
    S: Source + Write,
{
    /// Полим возможность записи данных
    pub(super) fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        // Мы сейчас в режиме ожидания записи данных?
        if self.waiting_write {
            // Тогда пробуем эти данные записать в сокет
            match self.source.write(buf) {
                Ok(n) => {
                    // Если данные не были записаны,
                    // значит сокет скорее всего уже просто закрыт
                    if n == 0 {
                        // Так что можно просто смело снять регистрацию
                        self.deregister()?;
                        // И сбросить флаг
                        self.waiting_write = false;
                    }

                    Poll::Ready(Ok(n))
                }
                // Если нам вернулся флаг, что данная операция потребует блокировки неблокирующего
                // сокета, но нам не надо его блокировать.
                // Вроде бы это вылезает при переполнениях буффера.
                //
                // For "writes", EWOULDBLOCK is saying "the first buffer hasn't been
                // completely sent and acknowledged yet -
                // you might want to hold off before you send anything else."
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => Poll::Pending,
                // Прочие ошибки
                Err(e) => Poll::Ready(Err(e)),
            }
        } else {
            // Если еще не было ожидания записи, тогда регистрируем
            // возможность чтения на сокете
            self.register(Interest::WRITABLE, cx.waker().clone())?;
            // Проставим флаг
            self.waiting_write = true;
            // Дальше ждем событий и пробуждения, может быть даже
            // сразу же и проснется данная футура если данные там есть
            return Poll::Pending;
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
