pub mod tcp;

mod strategy;
mod udp;

use crate::strategy::ProxyStrategy;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io;
use tokio_util::sync::CancellationToken;

/// Запуск проксирования
/// * `ct`: `CancellationToken`, по завершении которого задача проксирования будет остановлена
/// * `strategy`: стратегия проксирования
/// * `ls_map`: map соответствий адреса прослушивания в приложении и проксируемого удаленного адреса
pub async fn run_proxy(
    ct: CancellationToken,
    strategy: impl ProxyStrategy,
    ls_map: HashMap<SocketAddr, SocketAddr>,
) -> io::Result<()> {
    strategy.run(ct, ls_map).await
}
