use crate::{reactor::Reactor, Executor, ThreadPool};
use std::io;

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
    pub fn build_and_set_global(self) -> io::Result<()> {
        let task_threads = self.task_threads.unwrap_or_else(Self::default_thread_count);
        let task_tp = ThreadPool::new("task-worker".into(), task_threads);

        let blocking_threads = self
            .blocking_threads
            .unwrap_or_else(Self::default_thread_count);

        let blocking_tp = ThreadPool::new("blocking-worker".into(), blocking_threads);

        Executor::new(task_tp, blocking_tp).set_global();

        Reactor::new()?.set_global();

        Ok(())
    }

    fn default_thread_count() -> usize {
        num_cpus::get()
    }
}
