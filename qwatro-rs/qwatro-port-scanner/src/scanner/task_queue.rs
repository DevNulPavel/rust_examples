use crate::range::PortRange;
use std::collections::VecDeque;
use std::net::{IpAddr, SocketAddr};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

/// Запуск задачи на сканирование
/// * `ct`: `CancellationToken`, при отмене которого произойдет закрытие канала очереди задач и
/// завершение работы
/// * `task_queue_rx`: принимающая часть канала очереди задач
/// * `ip`: IP-адрес, на котором будут выполняться задачи сканирования
/// * `port_range`: диапазон портов, на которых будут выполняться задачи сканирования
pub fn run(
    ct: CancellationToken,
    mut task_queue_rx: mpsc::UnboundedReceiver<oneshot::Sender<Option<SocketAddr>>>,
    ip: IpAddr,
    port_range: PortRange,
) {
    tokio::spawn(async move {
        // Очередь задач сканирования на выполнение
        let mut task_queue = port_range
            .into_iter()
            .map(|p| SocketAddr::new(ip, p))
            .collect::<VecDeque<_>>();

        loop {
            tokio::select! {
                // В случае отмены CancellationToken, выходим из функции и закрываем канал очереди.
                // Воркеры увидят, что канал закрыт и завершат работу
                _ = ct.cancelled() => break,
                Some(tx) = task_queue_rx.recv() => {
                    let task = task_queue.pop_front();
                    let ended = task.is_none();
                    let _ = tx.send(task);
                    if ended {
                        log::debug!("port scan tasks queue ended");
                        break;
                    }
                }
            }
        }
    });
}
