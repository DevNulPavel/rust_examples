#![deny(missing_debug_implementations, rust_2018_idioms)]
// Запрет отсутствия реализации Debug у типов

use atomic_waker::AtomicWaker;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct WaitGroup {
    inner: Arc<Inner>,
}

impl Default for WaitGroup {
    fn default() -> Self {
        Self {
            inner: Arc::new(Inner::new()),
        }
    }
}

impl WaitGroup {
    /// Creates a new `WaitGroup`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new worker.
    pub fn worker(&self) -> Worker {
        // Увеличиваем атомарный счетчик заранее перед созданием
        self.inner.count.fetch_add(1, Ordering::Relaxed);
        Worker {
            inner: self.inner.clone(),
        }
    }

    /// Ждем завершения всех воркеров, которые были созданы
    pub async fn wait(&mut self) {
        WaitGroupFuture::new(&self.inner).await
    }
}

/// Футура обертка для ожидания результатов
struct WaitGroupFuture<'a> {
    inner: &'a Arc<Inner>,
}

impl<'a> WaitGroupFuture<'a> {
    fn new(inner: &'a Arc<Inner>) -> Self {
        Self { inner }
    }
}

impl Future for WaitGroupFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Проверим сначала, может быть уже все и так готово
        if self.inner.count.load(Ordering::Relaxed) == 0 {
            return Poll::Ready(());
        }

        // Сохраняем waker для будущего пробуждения
        // Без атомарного контейнера пришлось бы городить Mutex для сохранение Waker и тд
        self.inner.waker.register(cx.waker());

        // Теперь еще раз проверяем, может быть уже все завершено
        match self.inner.count.load(Ordering::Relaxed) {
            0 => Poll::Ready(()),
            _ => Poll::Pending,
        }
    }
}

struct Inner {
    waker: AtomicWaker,
    count: AtomicUsize,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
            waker: AtomicWaker::new(),
        }
    }
}

/// A worker registered in a `WaitGroup`.
///
/// Refer to the [crate level documentation](crate) for details.
pub struct Worker {
    inner: Arc<Inner>,
}

impl Worker {
    /// Notify the `WaitGroup` that this worker has finished execution.
    pub fn done(self) {
        drop(self)
    }
}

impl Clone for Worker {
    /// Cloning a worker increments the primary reference count and returns a new worker for use in
    /// another task.
    fn clone(&self) -> Self {
        self.inner.count.fetch_add(1, Ordering::Relaxed);
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let count = self.inner.count.fetch_sub(1, Ordering::Relaxed);
        // Это самый последний был воркер - отсылаем сообщение о пробуждении последнему сохраненному вейкеру
        if count == 1 {
            if let Some(waker) = self.inner.waker.take() {
                waker.wake();
            }
        }
    }
}

impl fmt::Debug for WaitGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.inner.count.load(Ordering::Relaxed);
        f.debug_struct("WaitGroup").field("count", &count).finish()
    }
}

impl fmt::Debug for Worker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.inner.count.load(Ordering::Relaxed);
        f.debug_struct("Worker").field("count", &count).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_wait_group() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        rt.block_on(async {
            let mut wg = WaitGroup::new();

            for _ in 0..5 {
                let worker = wg.worker();

                tokio::spawn(async {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    worker.done();
                });
            }

            wg.wait().await;
        });
    }

    #[test]
    fn test_wait_group_reuse() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        rt.block_on(async {
            let mut wg = WaitGroup::new();

            for _ in 0..5 {
                let worker = wg.worker();

                tokio::spawn(async {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    worker.done();
                });
            }

            wg.wait().await;

            let worker = wg.worker();

            tokio::spawn(async {
                tokio::time::sleep(Duration::from_secs(5)).await;
                worker.done();
            });

            wg.wait().await;
        });
    }

    #[test]
    fn test_worker_clone() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        rt.block_on(async {
            let mut wg = WaitGroup::new();

            for _ in 0..5 {
                let worker = wg.worker();

                tokio::spawn(async {
                    let nested_worker = worker.clone();
                    tokio::spawn(async {
                        nested_worker.done();
                    });
                    worker.done();
                });
            }

            wg.wait().await;
        });
    }
}
