mod inner;
mod queue;

use self::inner::Inner;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Job for worker
pub type Job = Box<dyn Fn() + Send>;

#[derive(Clone)]
pub struct ThreadPool(Arc<Inner>);

impl ThreadPool {
    pub fn new(name: String, thread_count: usize) -> Self {
        Self(Inner::new(name, thread_count))
    }

    pub fn spawn(&self, job: impl Fn() + Send + 'static) {
        self.0.spawn(job)
    }

    pub fn join(&self) -> Result<(), JoinError> {
        self.0.join()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("detected panicked threads while join: {0}")]
pub struct JoinError(u32);
