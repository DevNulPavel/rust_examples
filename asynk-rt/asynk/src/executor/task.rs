use super::{get_global_executor, handle::JoinHandle};
use futures::{channel::oneshot, FutureExt};
use parking_lot::Mutex;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake},
};

////////////////////////////////////////////////////////////////////////////////

/// Для удобства сокращенное имя таски
type TaskFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

////////////////////////////////////////////////////////////////////////////////

/// Непосредственно задача на исполнение
pub(super) struct Task<T, W> {
    /// Исполняемая футура
    fut: Mutex<Option<TaskFuture<T>>>,

    /// Канал для отдачи результата
    out_tx: Mutex<Option<oneshot::Sender<T>>>,

    /// PhantomData чисто для сохранения типа Waker
    _waker: PhantomData<W>,
}

impl<T, W> Task<T, W> {
    /// Создание новой задачи, вернет Arc + Join для ожидания
    pub(super) fn new(fut: impl Future<Output = T> + Send + 'static) -> (Arc<Self>, JoinHandle<T>)
    where
        T: Send + 'static,
    {
        // Обернем футуру в Pin + переместим со стека в кучу
        let fut = Box::pin(fut);

        // Канал для результата
        let (tx, rx) = oneshot::channel();

        // Создаем непосредственно задачу
        let task = Arc::new(Task {
            // Футура
            fut: Mutex::new(Some(fut)),
            // Канал
            out_tx: Mutex::new(Some(tx)),
            // Сохраняем тип Waker
            _waker: PhantomData,
        });

        (task, JoinHandle::new(rx))
    }

    // Оповещение, что задача отработала с результатом успешно
    fn ready(&self, output: T) {
        // Если у нас уже нету канала, то значит результат уже был получен кем-то
        if let Some(out_tx) = self.out_tx.lock().take() {
            out_tx.send(output).ok();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(super) struct SpawnedTaskWaker;

impl<T> Wake for Task<T, SpawnedTaskWaker>
where
    T: Send + 'static,
{
    fn wake(self: Arc<Self>) {
        get_global_executor().task_thread_pool().spawn(move || {
            let waker = self.clone().into();
            let mut cx = Context::from_waker(&waker);
            let mut lock = self.fut.lock();
            if let Some(mut fut) = lock.take() {
                match fut.poll_unpin(&mut cx) {
                    Poll::Ready(output) => self.ready(output),
                    Poll::Pending => *lock = Some(fut),
                };
            }
        });
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(super) struct BlockedOnTaskWaker;

impl<T> Wake for Task<T, BlockedOnTaskWaker>
where
    T: Send + 'static,
{
    fn wake(self: Arc<Self>) {
        let exec = get_global_executor();

        exec.task_thread_pool().spawn(move || {
            let waker = self.clone().into();
            let mut cx = Context::from_waker(&waker);
            let mut lock = self.fut.lock();
            if let Some(mut fut) = lock.take() {
                match fut.poll_unpin(&mut cx) {
                    Poll::Ready(output) => {
                        self.ready(output);
                        exec.unpark_blocked_thread();
                    }
                    Poll::Pending => *lock = Some(fut),
                };
            }
        });
    }
}
