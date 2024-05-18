pub mod tcp;

use crate::ScanType;
use async_trait::async_trait;
use std::net::SocketAddr;

/// Стратегия сканирования адреса
#[async_trait]
pub trait ScanStrategy {
    /// Просканировать адрес
    async fn scan(&self, addr: SocketAddr) -> bool;
    /// Тип сканирования
    fn scan_type(&self) -> ScanType;
}
