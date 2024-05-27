use crate::{
    executor::{try_set_global_executor, Executor},
    reactor::{try_set_global_reactor, Reactor},
    tp::ThreadPool,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("already initialized")]
    AlreadyInitialized,

    #[error("already initialized")]
    IO(#[from] std::io::Error),
}

////////////////////////////////////////////////////////////////////////////////

/// Билдер нашего рантайма
#[derive(Default)]
pub struct AsynkBuilder {
    /// Количество потоков
    task_threads: Option<usize>,

    /// Количество блокирующих потоков
    blocking_threads: Option<usize>,
}

impl AsynkBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    /// Количество потоков задач
    pub fn task_threads(mut self, val: usize) -> Self {
        let val = if val == 0 {
            Self::default_thread_count()
        } else {
            val
        };

        self.task_threads = Some(val);
        self
    }

    /// Количество блокирующих потоков
    pub fn blocking_threads(mut self, val: usize) -> Self {
        let val = if val == 0 {
            Self::default_thread_count()
        } else {
            val
        };

        self.blocking_threads = Some(val);
        self
    }

    /// Создаем рантайм и устанавливаем его как глобальный
    pub fn build_and_set_global(self) -> Result<(), BuildError> {
        let task_threads = self.task_threads.unwrap_or_else(Self::default_thread_count);
        let task_tp = ThreadPool::new("task-worker".into(), task_threads);

        let blocking_threads = self
            .blocking_threads
            .unwrap_or_else(Self::default_thread_count);

        let blocking_tp = ThreadPool::new("blocking-worker".into(), blocking_threads);

        // TODO: Пока в тестовом примере игнорим просто ошибку
        try_set_global_executor(Executor::new(task_tp, blocking_tp))
            .map_err(|_| BuildError::AlreadyInitialized)?;

        // TODO: Пока в тестовом примере игнорим просто ошибку
        try_set_global_reactor(Reactor::new()?).map_err(|_| BuildError::AlreadyInitialized)?;

        Ok(())
    }

    fn default_thread_count() -> usize {
        num_cpus::get()
    }
}
