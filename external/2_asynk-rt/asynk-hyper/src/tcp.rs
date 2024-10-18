use futures::{AsyncRead, AsyncWrite, Stream, StreamExt};
use hyper::rt::{Read, ReadBufCursor, Write};
use std::{
    io::Result,
    mem::MaybeUninit,
    net::SocketAddr,
    pin::Pin,
    task::{ready, Context, Poll},
};

pub struct TcpListener(asynk::net::TcpListener);

impl TcpListener {
    pub fn bind(addr: SocketAddr) -> Result<Self> {
        Ok(Self(asynk::net::TcpListener::bind(addr)?))
    }

    pub fn accept(self) -> Accept {
        Accept(self.0.accept())
    }
}

pub struct Accept(asynk::net::Accept);

impl Stream for Accept {
    type Item = Result<(TcpStream, SocketAddr)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Some(res) = ready!(self.0.poll_next_unpin(cx)) else {
            return Poll::Ready(None);
        };

        let (stream, addr) = res?;
        let stream = stream.into();

        Poll::Ready(Some(Ok((stream, addr))))
    }
}

/// TcpStream adapter for `hyper`
pub struct TcpStream(asynk::net::TcpStream);

impl From<asynk::net::TcpStream> for TcpStream {
    fn from(stream: asynk::net::TcpStream) -> Self {
        Self(stream)
    }
}

impl Read for TcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: ReadBufCursor<'_>,
    ) -> Poll<Result<()>> {
        unsafe {
            let b = buf.as_mut();
            let b = &mut *(b as *mut [MaybeUninit<u8>] as *mut [u8]);
            let n = ready!(Pin::new(&mut self.0).poll_read(cx, b))?;
            buf.advance(n);
        };

        Poll::Ready(Ok(()))
    }
}

impl Write for TcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}
