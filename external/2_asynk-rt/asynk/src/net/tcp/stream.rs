use crate::reactor::IoHandleOwned;
use futures::{AsyncRead, AsyncWrite};
use mio::net::TcpStream as MioTcpStream;
use std::{
    io::Result,
    net::{Shutdown, SocketAddr},
    pin::Pin,
    task::{Context, Poll},
};

////////////////////////////////////////////////////////////////////////////////

/// Обертка для TCP стрима mio
pub struct TcpStream(IoHandleOwned<MioTcpStream>);

impl TcpStream {
    // Создаем новый стрим для адреса
    pub fn connect(addr: SocketAddr) -> Result<Self> {
        // Создаем стрим
        let stream = MioTcpStream::connect(addr)?;

        // Оборачиваем его в хендл
        Ok(Self(IoHandleOwned::new(stream)))
    }
}

impl From<MioTcpStream> for TcpStream {
    fn from(stream: MioTcpStream) -> Self {
        Self(IoHandleOwned::new(stream))
    }
}

/// Реализуем асинхронное чтение
impl AsyncRead for TcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        // Так как IoHandleOwned реализует Unpin, то можно вполне
        // просто создать новый Pin без unsafe
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpStream {
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

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(self.0.source().shutdown(Shutdown::Both))
    }
}
