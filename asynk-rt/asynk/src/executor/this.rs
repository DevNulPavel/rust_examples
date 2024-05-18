////////////////////////////////////////////////////////////////////////////////

use super::{
    error::BlockOnError,
    handle::JoinError,
    task::{BlockedOnTaskWaker, SpawnedTaskWaker, Task},
};
use crate::{tp::ThreadPool, JoinHandle};
use futures::channel::oneshot;
use parking_lot::Mutex;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake},
    thread::{self, Thread},
};

////////////////////////////////////////////////////////////////////////////////

/// Исполнитель кода
pub(crate) struct Executor {
    /// Поток, который сейчас задействован в работе над block_on
    block_on_thr: Mutex<Option<Thread>>,

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
            block_on_thr: Mutex::new(None),
        }
    }

    pub(crate) fn task_thread_pool(&self) -> &ThreadPool {
        &self.task_tp
    }

    pub(crate) fn blocking_task_thread_pool(&self) -> &ThreadPool {
        &self.blocking_tp
    }

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
        // Пробуем взять блокировку, если кто-то другой уже испольует - вернем ошибку.
        {
            let mut lock = self.block_on_thr.lock();

            // Если никто у нас сейчас другой не испольует блокировку - установим ее
            if lock.is_none() {
                // Записываем ту
                *lock = Some(thread::current());
            } else {
                // Иначе вернем ошибку
                return Err(BlockOnError::AlreadyBlocked);
            }
        }

        // Создаем задачу и join
        let (task, mut jh) = Task::<T, BlockedOnTaskWaker>::new(fut);

        task.clone().wake();

        let main_waker = Arc::clone(&task).into();

        let mut cx = Context::from_waker(&main_waker);

        let mut jh = Pin::new(&mut jh);

        loop {
            // Check if main task is ready
            if let Poll::Ready(res) = jh.as_mut().poll(&mut cx) {
                return Ok(res?);
            }

            // Park this thread until main task become ready
            thread::park();
        }
    }

    pub(crate) fn spawn<T>(&self, fut: impl Future<Output = T> + Send + 'static) -> JoinHandle<T>
    where
        T: Send + 'static,
    {
        let (task, jh) = Task::<T, SpawnedTaskWaker>::new(fut);

        // Wake the task so that it starts trying to complete
        task.wake();
        jh
    }

    pub(crate) fn spawn_blocking<T>(&self, f: impl Fn() -> T + Send + 'static) -> JoinHandle<T>
    where
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        let tx = Mutex::new(Some(tx));
        self.blocking_tp.spawn(move || {
            let out = f();
            if let Some(tx) = tx.lock().take() {
                tx.send(out).ok();
            };
        });

        JoinHandle::new(rx)
    }

    pub(super) fn unpark_blocked_thread(&self) {
        self.block_on_thr
            .lock()
            .take()
            .expect("block on thread is not set")
            .unpark();
    }
}
