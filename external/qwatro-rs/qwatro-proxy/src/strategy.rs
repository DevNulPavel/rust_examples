use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io;
use tokio_util::sync::CancellationToken;

/// Map соответствий адреса прослушивания в приложении и проксируемого удаленного адреса
pub type HostToServerMap = HashMap<SocketAddr, SocketAddr>;

/// Стратегия проксирования
#[async_trait]
pub trait ProxyStrategy {
    /// Запуск задачи проксирования
    /// * `ct`: `CancellationToken`, по завершении которого задача проксирования будет остановлена
    /// * `ls_map`: map соответствий адреса прослушивания в приложении и проксируемого удаленного адреса
    async fn run(&self, ct: CancellationToken, hs_map: HostToServerMap) -> io::Result<()>;
}
