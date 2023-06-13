use super::{
    inner,
    pool::{
        BuildError, Counters, Edeltraud, EdeltraudLocal, Handle, Job, JobUnit, Shutdown, Stats,
    },
};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread,
    time::Instant,
};

////////////////////////////////////////////////////////////////////////

pub struct Builder {
    worker_threads: Option<usize>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Создаем билдер с нуля
    pub fn new() -> Builder {
        Builder {
            worker_threads: None,
        }
    }

    /// Задаем количество потоков исполнения.
    pub fn worker_threads(&mut self, value: usize) -> &mut Self {
        self.worker_threads = Some(value);
        self
    }

    /// Создаем пул потоков.
    /// Тип U должен реализовать трейт Job + поддержку конвертации из JobUnit с параметрами J, 
    /// которые в свою очередь должны реализовать Send для перекидывания между потоками + 
    /// содержать в себе лишь 'static ссылки.
    pub fn build<J, U>(&mut self) -> Result<Edeltraud<J>, BuildError>
    where
        U: Job + From<JobUnit<J, J>>,
        J: Send + 'static,
    {
        // Получаем количество потоков исполнения, или просто берем количество потоков
        // доступное по количеству ядер.
        let worker_threads = self.worker_threads.unwrap_or_else(num_cpus::get);

        // Кидаем ошибку если потоков почему-то всего-то 0
        if worker_threads == 0 {
            return Err(BuildError::ZeroWorkerThreadsCount);
        }

        let counters = Arc::new(Counters::default());
        
        let inner: Arc<inner::Inner<J>> =
            Arc::new(inner::Inner::new(worker_threads, counters.clone())?);

        let mut maybe_error = Ok(());
        let mut workers = Vec::with_capacity(worker_threads);
        let workers_sync: Arc<(Mutex<Option<Arc<Vec<_>>>>, Condvar)> =
            Arc::new((Mutex::new(None), Condvar::new()));
        for worker_index in 0..worker_threads {
            let inner = inner.clone();
            let workers_sync = workers_sync.clone();
            let counters = counters.clone();
            let maybe_join_handle = thread::Builder::new()
                .name(format!("edeltraud worker {}", worker_index))
                .spawn(move || {
                    let threads = 'outer: loop {
                        let Ok(mut workers_sync_lock) =
                            workers_sync.0.lock() else { return; };
                        loop {
                            if let Some(threads) = workers_sync_lock.as_ref() {
                                break 'outer threads.clone();
                            }
                            let Ok(next_workers_sync_lock) =
                                workers_sync.1.wait(workers_sync_lock) else { return; };
                            workers_sync_lock = next_workers_sync_lock;
                            if inner.is_terminated() {
                                return;
                            }
                        }
                    };

                    let mut worker_thread_pool = EdeltraudLocal {
                        handle: Handle {
                            inner: inner.clone(),
                            threads,
                        },
                        stats: Stats {
                            counters,
                            ..Default::default()
                        },
                    };
                    while let Some(job) =
                        inner.acquire_job(worker_index, &mut worker_thread_pool.stats)
                    {
                        let now = Instant::now();
                        let job_unit = JobUnit {
                            handle: worker_thread_pool.handle.clone(),
                            job,
                        };
                        let unit = U::from(job_unit);
                        unit.run();
                        worker_thread_pool.stats.job_run_time += now.elapsed();
                        worker_thread_pool.stats.job_run_count += 1;
                    }
                })
                .map_err(BuildError::WorkerSpawn);
            match maybe_join_handle {
                Ok(join_handle) => workers.push(join_handle),
                Err(error) => maybe_error = Err(error),
            }
        }

        let threads: Arc<Vec<_>> = Arc::new(
            workers
                .iter()
                .map(|handle| handle.thread().clone())
                .collect(),
        );
        if let Ok(mut workers_sync_lock) = workers_sync.0.lock() {
            *workers_sync_lock = Some(threads.clone());
            workers_sync.1.notify_all();
        }

        let thread_pool = Edeltraud {
            inner,
            threads,
            workers,
            shutdown: Arc::new(Shutdown {
                mutex: Mutex::new(false),
                condvar: Condvar::new(),
            }),
        };

        maybe_error?;
        log::debug!(
            "Edeltraud::new() success with {} workers",
            thread_pool.threads.len()
        );
        Ok(thread_pool)
    }
}
