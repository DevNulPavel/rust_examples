mod task;

pub(crate) mod handle;

////////////////////////////////////////////////////////////////////////////////

use self::task::{BlockedOnTaskWaker, SpawnedTaskWaker, Task};
use crate::{tp::ThreadPool, JoinHandle};
use futures::channel::oneshot;
use parking_lot::Mutex;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll, Wake},
    thread::{self, Thread},
};

////////////////////////////////////////////////////////////////////////////////

static EXECUTOR: OnceLock<Executor> = OnceLock::new();

////////////////////////////////////////////////////////////////////////////////

pub struct Executor {
    task_tp: ThreadPool,
    blocking_tp: ThreadPool,
    block_on_thr: Mutex<Option<Thread>>,
}

impl Executor {
    pub fn new(task_tp: ThreadPool, blocking_tp: ThreadPool) -> Self {
        Self {
            task_tp,
            blocking_tp,
            block_on_thr: Mutex::new(None),
        }
    }

    pub fn get() -> &'static Executor {
        EXECUTOR.get().expect("executor is not set")
    }

    pub fn set_global(self) {
        EXECUTOR.set(self).ok();
    }

    pub fn block_on<T>(
        &self,
        fut: impl Future<Output = T> + Send + 'static,
    ) -> Result<T, BlockOnError>
    where
        T: Send + 'static,
    {
        *self.block_on_thr.lock() = Some(thread::current());

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

    pub fn spawn<T>(&self, fut: impl Future<Output = T> + Send + 'static) -> JoinHandle<T>
    where
        T: Send + 'static,
    {
        let (task, jh) = Task::<T, SpawnedTaskWaker>::new(fut);

        // Wake the task so that it starts trying to complete
        task.wake();
        jh
    }

    pub fn spawn_blocking<T>(&self, f: impl Fn() -> T + Send + 'static) -> JoinHandle<T>
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

    fn unpark_blocked_thread(&self) {
        self.block_on_thr
            .lock()
            .take()
            .expect("block on thread is not set")
            .unpark();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockOnError {
    #[error("join error: {0}")]
    Join(#[from] handle::JoinError),
}
