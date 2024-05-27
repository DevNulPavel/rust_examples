use crate::{
    builder::Builder,
    pool::{job, job_async, AsyncJob, Computation, Job, JobUnit},
};
use std::{
    sync::{atomic, mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

///////////////////////////////////////////////////////////////////////////////////////////

struct SleepJob;

impl Computation for SleepJob {
    type Output = ();

    fn run(self) {
        thread::sleep(Duration::from_millis(100));
    }
}

/////////////////////////////////////////////////

#[test]
fn basic() {
    // Создаем систему
    let edeltraud = Builder::new()
        .worker_threads(4)
        .build::<_, JobUnit<_, _>>()
        .unwrap();

    // Получаем хендл пула потоков
    let pool = edeltraud.handle();

    // Создаем рантайм tokio
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    // Получаем текущее время
    let now = Instant::now();

    // Запускаем асинхронный рантайм в работу
    runtime.block_on(async move {
        // Буфер для задач
        let mut tasks = Vec::with_capacity(16);

        // Создаем футуры с задачами
        for _ in 0..16 {
            // Футура
            let f = async {
                // Создаем асинхронную задачу на пуле, которая просто спит
                job_async(&pool, SleepJob).unwrap().await
            };

            // Сохраняем задачу
            tasks.push(f);
        }

        // Ждем завершения всех задач на пуле потоков
        let _: Vec<()> = futures::future::try_join_all(tasks).await.unwrap();
    });

    // Делаем замер времени исполнения
    let elapsed = now.elapsed().as_secs_f64();
    assert!(
        elapsed >= 0.4,
        "elapsed expected to be >= 0.4, but it is {elapsed:?}"
    );
    assert!(
        elapsed < 0.5,
        "elapsed expected to be < 0.5, but it is {elapsed:?}"
    );
}

///////////////////////////////////////////////////////////////////////////////////////////

enum SleepJobRec {
    Master { sync_tx: mpsc::Sender<WorkComplete> },
    Slave { tx: mpsc::Sender<WorkComplete> },
}

/////////////////////////////////////////////////

struct WorkComplete;

/////////////////////////////////////////////////

struct SleepJobRecUnit<J>(JobUnit<J, SleepJobRec>);

impl<J> From<JobUnit<J, SleepJobRec>> for SleepJobRecUnit<J> {
    fn from(job_unit: JobUnit<J, SleepJobRec>) -> Self {
        Self(job_unit)
    }
}

impl<J> Job for SleepJobRecUnit<J>
where
    J: From<SleepJobRec>,
{
    fn run(self) {
        match self.0.job {
            SleepJobRec::Master { sync_tx } => {
                let (tx, rx) = mpsc::channel();
                for _ in 0..4 {
                    self.0
                        .handle
                        .spawn(SleepJobRec::Slave { tx: tx.clone() })
                        .unwrap();
                }
                for _ in 0..4 {
                    let WorkComplete = rx.recv().unwrap();
                }
                sync_tx.send(WorkComplete).ok();
            }
            SleepJobRec::Slave { tx } => {
                thread::sleep(Duration::from_millis(400));
                tx.send(WorkComplete).unwrap();
            }
        }
    }
}

/////////////////////////////////////////////////

#[test]
fn recursive_spawn() {
    let edeltraud = Builder::new()
        .worker_threads(5)
        .build::<_, SleepJobRecUnit<_>>()
        .unwrap();
    let pool = edeltraud.handle();
    let now = Instant::now();

    let (sync_tx, sync_rx) = mpsc::channel();
    job(&pool, SleepJobRec::Master { sync_tx }).unwrap();
    let WorkComplete = sync_rx.recv().unwrap();

    let elapsed = now.elapsed().as_secs_f64();
    assert!(
        elapsed >= 0.4,
        "elapsed expected to be >= 0.4, but it is {elapsed:?}"
    );
    assert!(
        elapsed < 0.5,
        "elapsed expected to be < 0.5, but it is {elapsed:?}"
    );
}

// multilayer_job

struct WrappedSleepJob(AsyncJob<SleepJob>);

impl From<AsyncJob<SleepJob>> for WrappedSleepJob {
    fn from(async_job: AsyncJob<SleepJob>) -> WrappedSleepJob {
        WrappedSleepJob(async_job)
    }
}

struct WrappedSleepJobUnit<J>(JobUnit<J, WrappedSleepJob>);

impl<J> From<JobUnit<J, WrappedSleepJob>> for WrappedSleepJobUnit<J> {
    fn from(job_unit: JobUnit<J, WrappedSleepJob>) -> Self {
        Self(job_unit)
    }
}

impl<J> Job for WrappedSleepJobUnit<J> {
    fn run(self) {
        let WrappedSleepJob(sleep_job) = self.0.job;
        let job_unit = JobUnit {
            handle: self.0.handle,
            job: sleep_job,
        };
        job_unit.run()
    }
}

#[test]
fn multilayer_job() {
    let edeltraud = Builder::new()
        .worker_threads(4)
        .build::<_, WrappedSleepJobUnit<_>>()
        .unwrap();
    let pool = edeltraud.handle();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let now = Instant::now();
    runtime.block_on(async move {
        let mut tasks = Vec::new();
        for _ in 0..16 {
            tasks.push(async { job_async(&pool, SleepJob).unwrap().await });
        }
        let _: Vec<()> = futures::future::try_join_all(tasks).await.unwrap();
    });
    let elapsed = now.elapsed().as_secs_f64();
    assert!(
        elapsed >= 0.4,
        "elapsed expected to be >= 0.4, but it is {elapsed:?}"
    );
    assert!(
        elapsed < 0.55,
        "elapsed expected to be < 0.55, but it is {elapsed:?}"
    );
}

///////////////////////////////////////////////////////////////////////////////////////////

struct SleepJobValue(isize);

impl Computation for SleepJobValue {
    type Output = isize;

    fn run(self) -> Self::Output {
        thread::sleep(Duration::from_millis(400));
        self.0
    }
}

#[test]
fn async_job() {
    let edeltraud = Builder::new()
        .worker_threads(4)
        .build::<_, JobUnit<_, _>>()
        .unwrap();
    let pool = edeltraud.handle();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let now = Instant::now();
    let value = runtime
        .block_on(job_async(&pool, SleepJobValue(144)).unwrap())
        .unwrap();
    let elapsed = now.elapsed().as_secs_f64();
    assert!(
        elapsed >= 0.4,
        "elapsed expected to be >= 0.4, but it is {elapsed:?}"
    );
    assert!(
        elapsed < 0.5,
        "elapsed expected to be < 0.5, but it is {elapsed:?}"
    );
    assert_eq!(value, 144);
}

// small_stress_job

#[test]
fn small_stress_job() {
    const JOBS_COUNT: usize = 256;
    const SUBJOBS_COUNT: usize = 1024;

    let shared_counter = Arc::new(atomic::AtomicUsize::new(0));

    struct StressJob {
        shared_counter: Arc<atomic::AtomicUsize>,
        allow_rec: bool,
    }

    struct StressJobUnit<J>(JobUnit<J, StressJob>);

    impl<J> From<JobUnit<J, StressJob>> for StressJobUnit<J> {
        fn from(job_unit: JobUnit<J, StressJob>) -> Self {
            Self(job_unit)
        }
    }

    impl<J> Job for StressJobUnit<J>
    where
        J: From<StressJob>,
    {
        fn run(self) {
            if self.0.job.allow_rec {
                for _ in 0..SUBJOBS_COUNT {
                    self.0
                        .handle
                        .spawn(StressJob {
                            shared_counter: self.0.job.shared_counter.clone(),
                            allow_rec: false,
                        })
                        .unwrap();
                }
            } else {
                self.0
                    .job
                    .shared_counter
                    .fetch_add(1, atomic::Ordering::Relaxed);
            }
        }
    }

    let edeltraud = Builder::new()
        .worker_threads(4)
        .build::<_, StressJobUnit<_>>()
        .unwrap();
    let thread_pool = edeltraud.handle();

    for _ in 0..JOBS_COUNT {
        thread_pool
            .spawn(StressJob {
                shared_counter: shared_counter.clone(),
                allow_rec: true,
            })
            .unwrap();
    }

    while shared_counter.load(atomic::Ordering::Relaxed) < JOBS_COUNT * SUBJOBS_COUNT {
        thread::sleep(Duration::from_millis(50));
    }
}
