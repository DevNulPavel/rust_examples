use super::accept::AcceptFuture;
use crate::reactor::{IoHandle, IoHandleRef};
use mio::net::TcpListener as MioTcpListener;
use std::{io::Result, net::SocketAddr};

////////////////////////////////////////////////////////////////////////////////

/// Отдельная структура для TCP листнера, оборачивает `MioTcpListener`
pub struct TcpListener {
    listener: MioTcpListener,
}

impl TcpListener {
    /// Биндимся на определенный адрес
    pub fn bind(addr: SocketAddr) -> Result<Self> {
        Ok(TcpListener {
            listener: MioTcpListener::bind(addr)?,
        })
    }

    /// Создает футуру для получения нового соединения
    pub fn accept(&mut self) -> AcceptFuture {
        AcceptFuture {
            handle: IoHandle::new(&mut self.listener),
        }
    }
}
