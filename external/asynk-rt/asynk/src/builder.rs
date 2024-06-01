use std::num::NonZeroUsize;

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
    task_threads: Option<NonZeroUsize>,

    /// Количество блокирующих потоков
    blocking_threads: Option<NonZeroUsize>,
}

impl AsynkBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    /// Количество потоков задач
    pub fn task_threads(mut self, val: NonZeroUsize) -> Self {
        self.task_threads = Some(val);

        self
    }

    /// Количество блокирующих потоков
    pub fn blocking_threads(mut self, val: NonZeroUsize) -> Self {
        self.blocking_threads = Some(val);

        self
    }

    /// Создаем рантайм и устанавливаем его как глобальный
    pub fn build_and_set_global(self) -> Result<(), BuildError> {
        // От безысходности пусть хотя бы один поток будет
        let task_threads = self
            .task_threads
            .or_else(|| NonZeroUsize::new(num_cpus::get()))
            .unwrap_or_else(|| unsafe { NonZeroUsize::new_unchecked(1) });

        let task_tp = ThreadPool::new("task-worker".into(), task_threads);

        // От безысходности пусть хотя бы один поток будет
        let blocking_threads = self
            .blocking_threads
            .or_else(|| NonZeroUsize::new(num_cpus::get()))
            .unwrap_or_else(|| unsafe { NonZeroUsize::new_unchecked(1) });

        let blocking_tp = ThreadPool::new("blocking-worker".into(), blocking_threads);

        // TODO: Пока в тестовом примере игнорим просто ошибку
        try_set_global_executor(Executor::new(task_tp, blocking_tp))
            .map_err(|_| BuildError::AlreadyInitialized)?;

        // TODO: Пока в тестовом примере игнорим просто ошибку
        try_set_global_reactor(Reactor::new()?).map_err(|_| BuildError::AlreadyInitialized)?;

        Ok(())
    }
}
