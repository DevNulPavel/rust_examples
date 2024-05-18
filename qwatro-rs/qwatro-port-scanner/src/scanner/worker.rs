use crate::strategy::ScanStrategy;
use crate::ScanResult;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

/// Запуск воркера, который будет выполнять задачи сканирования
/// * `task_queue_tx`: канал очереди задачи, у которого можно запросить новые задачи. Если канал
/// вернет `None`, значит задачи закончились. При закрытии канала воркер также останавливает работу
/// * `scan_res_tx`: канал, в который будут посылаться результаты успешного сканирования
/// * `strategies`: стратегии сканирования. Если успешно удалось произвести сканирование по одной
/// из стратегий, то сканирование считается удачным
pub fn spawn(
    task_queue_tx: UnboundedSender<oneshot::Sender<Option<SocketAddr>>>,
    scan_res_tx: UnboundedSender<ScanResult>,
    strategies: Arc<Vec<Box<dyn ScanStrategy + Send + Sync>>>,
) {
    tokio::spawn(async move {
        loop {
            let strategies = Arc::clone(&strategies);
            let (tx, rx) = oneshot::channel();
            if task_queue_tx.send(tx).is_err() {
                // Канал очереди задач закрылся - скорее всего, был завершен предварительно.
                // В таком случае, воркеру делать больше нечего, завершаем работу
                break;
            };

            match rx.await {
                Ok(Some(addr)) => {
                    for s in strategies.iter() {
                        if s.scan(addr).await {
                            let _ = scan_res_tx.send(ScanResult {
                                addr,
                                ty: s.scan_type(),
                            });

                            break;
                        }
                    }
                }
                // В очереди кончились задачи, либо oneshot-канал закрылся, завершаем работу
                _ => break,
            };
        }
    });
}
