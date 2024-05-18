use crate::{reactor::Reactor, Executor, ThreadPool};
use std::io;

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct AsynkBuilder {
    task_threads: Option<usize>,
    blocking_threads: Option<usize>,
}

impl AsynkBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn task_threads(mut self, val: usize) -> Self {
        let val = if val == 0 {
            Self::default_thread_count()
        } else {
            val
        };

        self.task_threads = Some(val);
        self
    }

    pub fn blocking_threads(mut self, val: usize) -> Self {
        let val = if val == 0 {
            Self::default_thread_count()
        } else {
            val
        };

        self.blocking_threads = Some(val);
        self
    }

    pub fn build(self) -> io::Result<()> {
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
