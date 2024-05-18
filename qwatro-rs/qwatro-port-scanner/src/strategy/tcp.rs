use crate::strategy::ScanStrategy;
use crate::ScanType;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;

const DEFAULT_RESP_TIMEOUT: Duration = Duration::from_millis(300);

/// TCP-сканирование порта
pub struct TcpScanning {
    /// Максимальное время ожидания на установление подключения
    resp_timeout: Duration,
}

impl TcpScanning {
    pub fn new(resp_timeout: Option<Duration>) -> Self {
        Self {
            resp_timeout: resp_timeout.unwrap_or(DEFAULT_RESP_TIMEOUT),
        }
    }
}

#[async_trait]
impl ScanStrategy for TcpScanning {
    async fn scan(&self, addr: SocketAddr) -> bool {
        matches!(
            time::timeout(self.resp_timeout, TcpStream::connect(addr)).await,
            Ok(Ok(_))
        )
    }

    fn scan_type(&self) -> ScanType {
        ScanType::TCP
    }
}
