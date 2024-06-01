use super::inner::Inner;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////

/// Job for worker
pub type Job = Box<dyn FnOnce() + Send>;

////////////////////////////////////////////////////////////////////////////////

/// Ошибка ожидания задачи
#[derive(Debug, thiserror::Error)]
#[error("detected panicked threads while join: {0}")]
pub(crate) struct JoinError(u32);

////////////////////////////////////////////////////////////////////////////////

/// Непосредственно тредпул, можно без проблем его клонить и шарить между потоками.
#[derive(Clone)]
pub(crate) struct ThreadPool(Arc<Inner>);

impl ThreadPool {
    pub(crate) fn new(name: String, thread_count: usize) -> Self {
        Self(Inner::new(name, thread_count))
    }

    /// Запуск нужной нам назадачи в пуле
    pub(crate) fn spawn(&self, job: impl FnOnce() + Send + 'static) {
        self.0.spawn(job)
    }

    /// Ожидание завершения задач всех в пуле.
    pub(crate) fn join(self) -> Result<(), JoinError> {
        self.0.join()
    }
}
