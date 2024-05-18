use crate::range::PortRange;
use crate::scanner::PortScanner;
use crate::strategy::tcp::TcpScanning;
use crate::strategy::ScanStrategy;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

/// Builder сканера портов
///
/// # Example
/// ```
/// use qwatro_port_scanner::builder::PortScannerBuilder;
///
/// let scanner = PortScannerBuilder::new()
///     .ip("192.168.100.1".parse().unwrap())
///     .tcp(None)
///     .build();
/// ```
pub struct PortScannerBuilder {
    /// IP-адрес
    ip: IpAddr,
    /// Диапазон портов
    port_range: PortRange,
    /// Максимальное количество параллельно запущенных задач сканирования
    max_tasks: usize,
    /// Стратегии сканирования
    strategies: Vec<Box<dyn ScanStrategy + Send + Sync>>,
}

impl Default for PortScannerBuilder {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port_range: PortRange::ordered(1, u16::MAX).unwrap(),
            max_tasks: 500,
            strategies: vec![],
        }
    }
}

impl PortScannerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// IP-адрес сканируемого хоста
    pub fn ip(mut self, ip: IpAddr) -> Self {
        self.ip = ip;
        self
    }

    /// Диапазон портов
    pub fn port_range(mut self, port_range: PortRange) -> Self {
        self.port_range = port_range;
        self
    }

    /// Включить режим tcp-сканирования
    ///
    /// `respTimeout` - максимальное время ожидания установления подключения. Если `None` - будет
    /// взято значение по умолчанию
    pub fn tcp(mut self, resp_timeout: Option<Duration>) -> Self {
        self.strategies
            .push(Box::new(TcpScanning::new(resp_timeout)));
        self
    }

    /// Максимальное количество параллельно запущенных задач сканирования
    pub fn max_tasks(mut self, max_tasks: usize) -> Self {
        self.max_tasks = max_tasks;
        self
    }

    /// Создать `PortScanner`
    pub fn build(self) -> PortScanner {
        PortScanner::new(
            self.ip,
            self.port_range,
            self.max_tasks,
            Arc::new(self.strategies),
        )
    }
}
