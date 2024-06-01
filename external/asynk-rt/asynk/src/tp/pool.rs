use super::inner::Inner;
use std::{num::NonZeroUsize, sync::Arc};

////////////////////////////////////////////////////////////////////////////////

/// Job for worker
pub type Job = Box<dyn FnOnce() + Send>;

////////////////////////////////////////////////////////////////////////////////

/// Ошибка ожидания задачи
#[derive(Debug, thiserror::Error)]
#[error("detected panicked threads while join: {0}")]
pub(crate) struct JoinError(pub(crate) u32);

////////////////////////////////////////////////////////////////////////////////

/// Непосредственно тредпул, можно без проблем его клонить и шарить между потоками.
#[derive(Clone)]
pub(crate) struct ThreadPool(Arc<Inner>);

impl ThreadPool {
    pub(crate) fn new(name: String, thread_count: NonZeroUsize) -> Self {
        ThreadPool(Inner::new_arc(name, thread_count))
    }

    /// Запуск нужной нам назадачи в пуле
    pub(crate) fn spawn<J>(&self, job: J)
    where
        // Функция разового запуска, которую можем перемещать
        // из одного потока в другой и выполнять.
        // Этот функтор может содержать лишь статические ссылки.
        J: FnOnce() + Send + 'static,
    {
        self.0.spawn(job)
    }

    /// Ожидание завершения задач всех в пуле.
    /// Завершать работу не обязательно будет текущий поток, может быть кто-то другой.
    /// Он и будет обрабатывать ошибки.
    /// Если функция завершилась, значит пул потоков уже точно не работает.
    pub(crate) fn join(self) -> Result<(), JoinError> {
        self.0.join()
    }
}
