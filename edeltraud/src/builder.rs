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

        // Общие счетчики
        let counters = Arc::new(Counters::default());

        // Создаем Inner обработчик
        let inner: Arc<inner::Inner<J>> =
            Arc::new(inner::Inner::new(worker_threads, counters.clone())?);

        // Тип ошибки
        let mut maybe_error = Ok(());

        // Массив воркеров
        let mut workers = Vec::with_capacity(worker_threads);

        // CondVar для воркеров
        let workers_sync: Arc<(Mutex<Option<Arc<Vec<_>>>>, Condvar)> =
            Arc::new((Mutex::new(None), Condvar::new()));

        // Код создания
        for worker_index in 0..worker_threads {
            // Клоним распределитель для задач
            let inner = inner.clone();

            // Клоним CondVar для воркеров
            let workers_sync = workers_sync.clone();

            // Клоним счетчики для метрик
            let counters = counters.clone();

            // Создаем поток исполнения с определенным именем
            let maybe_join_handle = thread::Builder::new()
                .name(format!("edeltraud worker {}", worker_index))
                .spawn(move || {
                    // Получаем Arc на вектор потоков
                    let threads = 'outer: loop {
                        // Блокируемся на Mutex синхронизации
                        let Ok(mut workers_sync_lock) =
                            workers_sync.0.lock() else { return; };

                        loop {
                            // Есть ли потоки уже?
                            // Если есть - возвращаем из цикла вектор потоков
                            if let Some(threads) = workers_sync_lock.as_ref() {
                                break 'outer threads.clone();
                            }

                            // Если потоков еще нету, тогжа ждем на CondVar+Lock когда появятся.
                            // Получается некоторый аналог барьера своего рода.
                            let Ok(next_workers_sync_lock) =
                                workers_sync.1.wait(workers_sync_lock) else { return; };

                            // Возвращаем Lock назад для очередной итерации
                            workers_sync_lock = next_workers_sync_lock;

                            // Не была ли еще завершена работа?
                            if inner.is_terminated() {
                                return;
                            }
                        }
                    };

                    // Создаем пул потоков с потоками созданными
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

                    // Пробуем получить работу из менеджера
                    while let Some(job) =
                        inner.acquire_job(worker_index, &mut worker_thread_pool.stats)
                    {
                        // Замер времени
                        let now = Instant::now();

                        // Создаем задачу
                        let job_unit = JobUnit {
                            handle: worker_thread_pool.handle.clone(),
                            job,
                        };

                        // Создаем юнит
                        let unit = U::from(job_unit);

                        // Запускаем его в работу на текущем потоке исполнения
                        unit.run();

                        // Делаем замеры
                        worker_thread_pool.stats.job_run_time += now.elapsed();
                        worker_thread_pool.stats.job_run_count += 1;
                    }
                })
                .map_err(BuildError::WorkerSpawn);

            // Сохраняем запущенного воркера
            match maybe_join_handle {
                // Пушим воркера
                Ok(join_handle) => workers.push(join_handle),
                // Либо сохраняем ошибку создания
                Err(error) => maybe_error = Err(error),
            }
        }

        // Здесь мы складываем в синхронизацию созданные потоки исполнения.
        let threads: Arc<Vec<_>> = Arc::new(
            workers
                .iter()
                .map(|handle| handle.thread().clone())
                .collect(),
        );
        // Фактически с этого самого момента и начинается у нас исполнение задач после того,
        // как мы записали хендлы потоков в общуюу кучу.
        if let Ok(mut workers_sync_lock) = workers_sync.0.lock() {
            *workers_sync_lock = Some(threads.clone());
            workers_sync.1.notify_all();
        }

        // Создаем систему
        let thread_pool = Edeltraud {
            inner,
            threads,
            workers,
            shutdown: Arc::new(Shutdown {
                mutex: Mutex::new(false),
                condvar: Condvar::new(),
            }),
        };

        // Не было ли ошибок никаких?
        maybe_error?;

        log::debug!(
            "Edeltraud::new() success with {} workers",
            thread_pool.threads.len()
        );

        Ok(thread_pool)
    }
}
