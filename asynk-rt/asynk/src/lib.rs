pub mod net;

mod builder;
mod executor;
mod reactor;
mod tp;

////////////////////////////////////////////////////////////////////////////////

use executor::Executor;
use std::future::Future;

////////////////////////////////////////////////////////////////////////////////

pub use {
    builder::AsynkBuilder,
    executor::{handle::JoinHandle, BlockOnError},
    tp::ThreadPool,
};

////////////////////////////////////////////////////////////////////////////////

/// Runtime builder
pub fn builder() -> AsynkBuilder {
    AsynkBuilder::new()
}

/// Block current thread on the provided asynchronous task
pub fn block_on<T>(fut: impl Future<Output = T> + Send + 'static) -> Result<T, BlockOnError>
where
    T: Send + 'static,
{
    Executor::get().block_on(fut)
}

/// Spawn new asynchronous task
pub fn spawn<T>(fut: impl Future<Output = T> + Send + 'static) -> JoinHandle<T>
where
    T: Send + 'static,
{
    Executor::get().spawn(fut)
}

/// Spawn synchronous task on dedicated thread pool
pub fn spawn_blocking<T>(f: impl Fn() -> T + Send + 'static) -> JoinHandle<T>
where
    T: Send + 'static,
{
    Executor::get().spawn_blocking(f)
}
