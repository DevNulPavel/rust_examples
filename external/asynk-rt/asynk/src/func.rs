use crate::executor::get_global_executor;
use std::future::Future;

pub use super::{
    builder::AsynkBuilder,
    executor::{BlockOnError, JoinHandle},
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
    get_global_executor().block_on(fut)
}

/// Запускаем новую асинхронную задачу
pub fn spawn<T>(fut: impl Future<Output = T> + Send + 'static) -> JoinHandle<T>
where
    T: Send + 'static,
{
    // Получам глобальный исполнитель и запускаем задачу
    get_global_executor().spawn(fut)
}

/// Запускаем блокирующую задачу на пуле потоков + получаем возможность асинхронно дождаться результатов
pub fn spawn_blocking<T>(f: impl Fn() -> T + Send + 'static) -> JoinHandle<T>
where
    T: Send + 'static,
{
    // Получам глобальный исполнитель и запускаем задачу
    get_global_executor().spawn_blocking(f)
}
