/// Builder сканера
pub mod builder;
/// Ошибки сканера портов
pub mod error;
/// Диапазон сканирования портов
pub mod range;
/// Реализация сканирования портов
pub mod scanner;

/// Стратегии сканирования
mod strategy;

use std::net::SocketAddr;

/// Тип сканирования
#[derive(Debug)]
pub enum ScanType {
    TCP,
    UDP,
}

/// Результат сканирования
#[derive(Debug)]
pub struct ScanResult {
    pub addr: SocketAddr,
    pub ty: ScanType,
}
