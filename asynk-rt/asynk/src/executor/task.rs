use super::{get_global_executor, handle::JoinHandle};
use futures::{channel::oneshot, FutureExt};
use parking_lot::Mutex;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread::{self, Thread},
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
    waker: W,
}

impl<T, W> Task<T, W> {
    /// Создание новой задачи, вернет Arc + Join для ожидания
    pub(super) fn new(
        fut: impl Future<Output = T> + Send + 'static,
        waker: W,
    ) -> (Arc<Self>, JoinHandle<T>)
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
            waker,
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

/// Waker для запущенной фоновой задачи
pub(super) struct SpawnedTaskWaker;

////////////////////////////////////////////////////////////////////////////////

pub(super) struct BlockedOnTaskWaker(Mutex<Option<Thread>>);

impl BlockedOnTaskWaker {
    pub(super) fn new_current_thread() -> BlockedOnTaskWaker {
        BlockedOnTaskWaker(Mutex::new(Some(thread::current())))
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Реализуем стандартный интерфейс для пробуждения задач
impl<T> Wake for Task<T, SpawnedTaskWaker>
where
    T: Send + 'static,
{
    /// Реализация пробуждения
    fn wake(self: Arc<Self>) {
        // Получаем тредпул для обычных задач из глобального исполнителя
        let tp = get_global_executor().task_thread_pool();

        // Запускаем задачу на пуле потоков
        tp.spawn(move || {
            // Создаем клон текущего waker + превращаем его в стандартный тип
            // Что интересно, здесь не происходит никаких аллокаций больше,
            // тип Arc<dyn Wake> конвертируется в Waker внутри
            let waker: Waker = self.clone().into();

            // Создаем дополнительно контекст-обертку еще из waker
            let mut cx = Context::from_waker(&waker);

            // Блокируемся на Mutex футуры
            let mut lock = self.fut.lock();

            // Пробуем получить футуру из Mutex, оставив там None
            if let Some(mut fut) = lock.take() {
                // Если у нас есть все еще футура - полим ее
                match fut.poll_unpin(&mut cx) {
                    // Если завершилось успешно - оповещаем о завершении
                    Poll::Ready(output) => {
                        // Оповещаем о завершении
                        self.ready(output)
                    }
                    // Если задача еще не готова
                    Poll::Pending => {
                        // Тогда вернем футуру назад
                        *lock = Some(fut)
                    }
                };
            }
        });
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Реализуем стандартный интерфейс для пробуждения задач
impl<T> Wake for Task<T, BlockedOnTaskWaker>
where
    T: Send + 'static,
{
    fn wake(self: Arc<Self>) {
        // Получаем тредпул для обычных задач из глобального исполнителя
        let exec = get_global_executor();
        let tp = exec.task_thread_pool();

        // Запусаем задачу на пуле потоков, текущее пробуждение не будем блокиовать.
        tp.spawn(move || {
            // Создаем клон текущего waker + превращаем его в стандартный тип
            // Что интересно, здесь не происходит никаких аллокаций больше,
            // тип Arc<dyn Wake> конвертируется в Waker внутри
            let waker = self.clone().into();

            // Создаем дополнительно контекст-обертку еще из waker
            let mut cx = Context::from_waker(&waker);

            // Блокируемся на Mutex футуры
            let mut lock = self.fut.lock();

            // Пробуем получить футуру из Mutex, оставив там None
            if let Some(mut fut) = lock.take() {
                // Если у нас есть все еще футура - полим ее.
                // Следующий вызов полинга будет регистрироваться там где-то внутри.
                match fut.poll_unpin(&mut cx) {
                    // Если завершилось успешно - оповещаем о завершении
                    Poll::Ready(output) => {
                        // Оповещаем о завершении
                        self.ready(output);

                        // Здесь мы дополнительно при пробуждении еще
                        // разблокируем заблокированный поток исполнителя
                        if let Some(thread) = self.waker.0.lock().take() {
                            thread.unpark();
                        }
                    }
                    // Если задача еще не готова
                    Poll::Pending => {
                        // Тогда вернем футуру назад
                        *lock = Some(fut)
                    }
                };
            }
        });
    }
}
