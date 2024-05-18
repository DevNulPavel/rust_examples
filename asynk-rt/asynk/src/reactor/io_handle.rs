// use crate::reactor_global;
use super::Reactor;
use mio::{event::Source, Interest, Token};
use std::{
    io::{ErrorKind, Read, Result, Write},
    pin::Pin,
    task::{Context, Poll, Waker},
};

pub struct IoHandle<S>
where
    S: Source,
{
    source: S,
    waiting_read: bool,
    waiting_write: bool,
    token: Option<Token>,
}

impl<S> Unpin for IoHandle<S> where S: Source {}

impl<S> IoHandle<S>
where
    S: Source,
{
    pub fn new(source: S) -> Self {
        Self {
            source,
            waiting_read: false,
            waiting_write: false,
            token: None,
        }
    }

    pub fn source(&self) -> &S {
        &self.source
    }

    pub fn register(&mut self, interest: Interest, waker: Waker) -> Result<()> {
        match self.token {
            Some(token) => {
                Reactor::get().reregister(token, &mut self.source, interest, waker)?;
            }
            None => {
                let token = Reactor::get().register(&mut self.source, interest, waker)?;
                self.token = Some(token);
            }
        };

        Ok(())
    }

    pub fn deregister(&mut self) -> Result<()> {
        match self.token {
            Some(token) => {
                Reactor::get().deregister(token, &mut self.source)?;
                self.token = None;
                Ok(())
            }
            None => Ok(()),
        }
    }
}

impl<S> IoHandle<S>
where
    S: Source + Read,
{
    pub fn poll_read(
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
    pub fn poll_write(
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

    pub fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
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
