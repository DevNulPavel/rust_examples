pub mod net;

////////////////////////////////////////////////////////////////////////////////

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

/// Билдер для рантайма
pub fn builder() -> AsynkBuilder {
    AsynkBuilder::new()
}

/// Блокируем текущий поток до момента завершнеия асинхронной задачи
pub fn block_on<T>(fut: impl Future<Output = T> + Send + 'static) -> Result<T, BlockOnError>
where
    T: Send + 'static,
{
    // Получам глобальный исполнитель и запускаем задачу
    Executor::get().block_on(fut)
}

/// Запускаем новую асинхронную задачу
pub fn spawn<T>(fut: impl Future<Output = T> + Send + 'static) -> JoinHandle<T>
where
    T: Send + 'static,
{
    // Получам глобальный исполнитель и запускаем задачу
    Executor::get().spawn(fut)
}

/// Запускаем блокирующую задачу на пуле потоков + получаем возможность асинхронно дождаться результатов
pub fn spawn_blocking<T>(f: impl Fn() -> T + Send + 'static) -> JoinHandle<T>
where
    T: Send + 'static,
{
    // Получам глобальный исполнитель и запускаем задачу
    Executor::get().spawn_blocking(f)
}
