use super::{handle::JoinHandle, Executor};
use futures::{channel::oneshot, FutureExt};
use parking_lot::Mutex;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake},
};

type TaskFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub struct Task<T, W> {
    fut: Mutex<Option<TaskFuture<T>>>,
    out_tx: Mutex<Option<oneshot::Sender<T>>>,
    _waker: PhantomData<W>,
}

impl<T, W> Task<T, W> {
    pub fn new(fut: impl Future<Output = T> + Send + 'static) -> (Arc<Self>, JoinHandle<T>)
    where
        T: Send + 'static,
    {
        let fut = Box::pin(fut);

        let (tx, rx) = oneshot::channel();

        let task = Arc::new(Task {
            fut: Mutex::new(Some(Box::pin(fut))),
            out_tx: Mutex::new(Some(tx)),
            _waker: PhantomData,
        });

        (task, JoinHandle::new(rx))
    }

    fn ready(&self, output: T) {
        // If result channel is dropped, then probably task output was taken
        if let Some(out_tx) = self.out_tx.lock().take() {
            out_tx.send(output).ok();
        }
    }
}

pub struct SpawnedTaskWaker;

pub struct BlockedOnTaskWaker;

impl<T> Wake for Task<T, SpawnedTaskWaker>
where
    T: Send + 'static,
{
    fn wake(self: Arc<Self>) {
        Executor::get().task_tp.spawn(move || {
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

impl<T> Wake for Task<T, BlockedOnTaskWaker>
where
    T: Send + 'static,
{
    fn wake(self: Arc<Self>) {
        let exec = Executor::get();

        exec.task_tp.spawn(move || {
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
