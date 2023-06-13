use super::inner;
use futures::{channel::oneshot, Future};
use std::{
    io,
    pin::Pin,
    sync::{atomic, Arc, Condvar, Mutex},
    task::Poll,
    thread,
    time::{Duration, Instant},
};

////////////////////////////////////////////////////////////////////////

/// Трейт для непосредственно исполняемой работы, которая не возвращает никаких
/// резултатов из себя.
///
/// Трейт требует, чтобы была реализация Sized, так как методы принимают
/// конкретный self объект конкретного типа, а не по ссылке.
///
/// Читать про Object safety.
pub trait Job: Sized {
    fn run(self);
}

////////////////////////////////////////////////////////////////////////

pub struct JobUnit<J, G> {
    pub handle: Handle<J>,
    pub job: G,
}

////////////////////////////////////////////////////////////////////////

/// Трейт для какой-то задачи, которая может выполниться и выдать конкретный результат.
///
/// Трейт требует, чтобы была реализация Sized, так как методы принимают
/// конкретный self объект конкретного типа, а не по ссылке.
///
/// Читать про Object safety.
pub trait Computation: Sized {
    type Output;

    fn run(self) -> Self::Output;
}

////////////////////////////////////////////////////////////////////////

/// Непосредственно реализация самого пула.
///
pub struct Edeltraud<J> {
    /// Внутренние воркеры и обработчики
    pub(super) inner: Arc<inner::Inner<J>>,

    /// Непосредственно хендлы пула потоков
    pub(super) threads: Arc<Vec<thread::Thread>>,

    /// Массив join для пула потоков
    pub(super) workers: Vec<thread::JoinHandle<()>>,

    /// Специальный объект-сигнал для оповещения о завершении работы.
    pub(super) shutdown: Arc<Shutdown>,
}

////////////////////////////////////////////////////////////////////////

pub struct Handle<J> {
    pub(super) inner: Arc<inner::Inner<J>>,
    pub(super) threads: Arc<Vec<thread::Thread>>,
}

impl<J> Clone for Handle<J> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            threads: self.threads.clone(),
        }
    }
}

impl<J> Handle<J> {
    pub fn spawn<G>(&self, job: G) -> Result<(), SpawnError>
    where
        J: From<G>,
    {
        self.inner.spawn(job.into(), &self.threads)
    }
}

////////////////////////////////////////////////////////////////////////

pub(super)  struct Shutdown {
    pub(super) mutex: Mutex<bool>,
    pub(super) condvar: Condvar,
}

////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct Counters {
    pub spawn_total_count: atomic::AtomicUsize,
    pub spawn_touch_tag_collisions: atomic::AtomicUsize,
}

////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct Stats {
    pub acquire_job_time: Duration,
    pub acquire_job_count: usize,
    pub acquire_job_backoff_time: Duration,
    pub acquire_job_backoff_count: usize,
    pub acquire_job_thread_park_time: Duration,
    pub acquire_job_thread_park_count: usize,
    pub acquire_job_seg_queue_pop_time: Duration,
    pub acquire_job_seg_queue_pop_count: usize,
    pub acquire_job_taken_by_collisions: usize,
    pub job_run_time: Duration,
    pub job_run_count: usize,
    pub counters: Arc<Counters>,
}

impl<J> Edeltraud<J> {
    pub fn handle(&self) -> Handle<J> {
        Handle {
            inner: self.inner.clone(),
            threads: self.threads.clone(),
        }
    }

    pub fn shutdown_timeout(self, timeout: Duration) {
        self.inner.force_terminate(&self.threads);
        if let Ok(mut lock) = self.shutdown.mutex.lock() {
            let mut current_timeout = timeout;
            let now = Instant::now();
            while !*lock {
                match self.shutdown.condvar.wait_timeout(lock, current_timeout) {
                    Ok((next_lock, wait_timeout_result)) => {
                        lock = next_lock;
                        if wait_timeout_result.timed_out() {
                            log::info!("shutdown wait timed out");
                            break;
                        }
                        current_timeout -= now.elapsed();
                    }
                    Err(..) => {
                        log::error!("failed to wait on shutdown condvar, terminating immediately");
                        break;
                    }
                }
            }
        } else {
            log::error!("failed to lock shutdown mutex, terminating immediately");
        }
    }
}

impl<J> Drop for Edeltraud<J> {
    fn drop(&mut self) {
        log::debug!("Edeltraud::drop() invoked");
        self.inner.force_terminate(&self.threads);
        for join_handle in self.workers.drain(..) {
            join_handle.join().ok();
        }
        if let Ok(mut lock) = self.shutdown.mutex.lock() {
            *lock = true;
            self.shutdown.condvar.notify_all();
        } else {
            log::error!("failed to lock shutdown mutex on drop");
        }
    }
}

#[derive(Debug)]
pub enum BuildError {
    ZeroWorkerThreadsCount,
    TooBigWorkerThreadsCount,
    WorkerSpawn(io::Error),
}

#[derive(Debug)]
pub enum SpawnError {
    ThreadPoolGone,
}

pub struct AsyncJob<G>
where
    G: Computation,
{
    computation: G,
    result_tx: oneshot::Sender<G::Output>,
}

pub fn job<J, G>(thread_pool: &Handle<J>, job: G) -> Result<(), SpawnError>
where
    J: From<G>,
{
    thread_pool.spawn(job)
}

pub fn job_async<J, G>(
    thread_pool: &Handle<J>,
    computation: G,
) -> Result<AsyncResult<G::Output>, SpawnError>
where
    J: From<AsyncJob<G>>,
    G: Computation,
{
    let (result_tx, result_rx) = oneshot::channel();
    let async_job = AsyncJob {
        computation,
        result_tx,
    };
    thread_pool.spawn(async_job)?;

    Ok(AsyncResult { result_rx })
}

impl<J, G> Job for JobUnit<J, AsyncJob<G>>
where
    G: Computation,
{
    fn run(self) {
        let output = self.job.computation.run();
        if let Err(_send_error) = self.job.result_tx.send(output) {
            log::warn!("async result channel dropped before job is finished");
        }
    }
}

pub(super) struct EdeltraudLocal<J> {
    pub(super) handle: Handle<J>,
    pub(super) stats: Stats,
}

impl<J> Drop for EdeltraudLocal<J> {
    fn drop(&mut self) {
        self.handle.inner.force_terminate(&self.handle.threads);
        log::info!(
            "EdeltraudLocal::drop on {:?}, stats: {:?}",
            thread::current(),
            self.stats
        );
    }
}

pub struct AsyncResult<T> {
    result_rx: oneshot::Receiver<T>,
}

impl<T> Future for AsyncResult<T> {
    type Output = Result<T, SpawnError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let result_rx = Pin::new(&mut this.result_rx);
        result_rx
            .poll(cx)
            .map_err(|oneshot::Canceled| SpawnError::ThreadPoolGone)
    }
}
