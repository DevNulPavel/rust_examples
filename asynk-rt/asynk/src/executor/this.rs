////////////////////////////////////////////////////////////////////////////////

use super::{
    error::BlockOnError,
    task::{BlockedOnTaskWaker, SpawnedTaskWaker, Task},
};
use crate::{tp::ThreadPool, JoinHandle};
use futures::channel::oneshot;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread,
};

////////////////////////////////////////////////////////////////////////////////

/// Исполнитель кода
pub(crate) struct Executor {
    /// Поток, который сейчас задействован в работе над block_on
    // block_on_thr: Mutex<Option<Thread>>,

    /// Пул потоков для задач обычных асинхронных
    task_tp: ThreadPool,

    /// Пул потоков для блокирующих задач
    blocking_tp: ThreadPool,
}

impl Executor {
    /// Новый исполнитель с пулами потоков
    pub(crate) fn new(task_tp: ThreadPool, blocking_tp: ThreadPool) -> Self {
        Self {
            task_tp,
            blocking_tp,
            // block_on_thr: Mutex::new(None),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    // Получаем пул потоков для задач
    pub(crate) fn task_thread_pool(&self) -> &ThreadPool {
        &self.task_tp
    }

    ////////////////////////////////////////////////////////////////////////////////

    // Получаем пул потоков для блокирующих задач
    pub(crate) fn blocking_task_thread_pool(&self) -> &ThreadPool {
        &self.blocking_tp
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Запускаем корневую футуру на которой будет заблокирован исполнитель.
    ///
    /// Поддерживается лишь одна такая футура за раз.
    pub(crate) fn block_on<T>(
        &self,
        fut: impl Future<Output = T> + Send + 'static,
    ) -> Result<T, BlockOnError>
    where
        T: Send + 'static,
    {
        // Создаем задачу и join
        let (task, mut jh) =
            Task::<T, BlockedOnTaskWaker>::new(fut, BlockedOnTaskWaker::new_current_thread());

        // Один раз вызываем wake для Arc задачи, может быть она сразу же окажется завершена +
        // тем самым мы запускаем полинг
        task.clone().wake();

        // Конвертируем без аллокаций наш Arc задачи в стандартный Waker тип.
        // Так можно делать благодаря реализации `Arc<dyn Wake>`.
        let main_waker: Waker = Arc::clone(&task).into();

        // Создаем контекст из нашего Waker
        let mut cx = Context::from_waker(&main_waker);

        // Дополнитеьно пинируем на стеке Join
        let mut jh = Pin::new(&mut jh);

        // Выполняем главный цикл работы ожидания завершения
        loop {
            // Периодически проверяем, не завершилась ли еще задача, проверяя канал на завершение и получение результата
            match jh.as_mut().poll(&mut cx) {
                // Готова задача
                Poll::Ready(res) => {asd
                    return Ok(res?);
                }
                Poll::Pending => {
                    // Спим, пока в канале что-то не появится в канале.
                    // Пробуждение будет внутри BlockedOnTaskWaker.
                    thread::park();
                }
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Запуска футуры фоновой
    pub(crate) fn spawn<T>(&self, fut: impl Future<Output = T> + Send + 'static) -> JoinHandle<T>
    where
        T: Send + 'static,
    {
        // Создадим задачу
        let (task, jh) = Task::<T, SpawnedTaskWaker>::new(fut, SpawnedTaskWaker);

        // Первичный запуск задачи, который триггерит полинг футуры
        task.wake();

        // Канал для ожидания завершения
        jh
    }

    ////////////////////////////////////////////////////////////////////////////////

    /// Запуск какой-то блокирующей задачи на пуле потоков
    pub(crate) fn spawn_blocking<T>(&self, f: impl Fn() -> T + Send + 'static) -> JoinHandle<T>
    where
        T: Send + 'static,
    {
        // Канал ожидания завершения задачи
        let (tx, rx) = oneshot::channel();

        // Запускаем на пуле потоков работу
        self.blocking_tp.spawn(move || {
            let out = f();

            // Отправляем результат если кто-то его еще ждет
            tx.send(out).ok();
        });

        JoinHandle::new(rx)
    }
}
