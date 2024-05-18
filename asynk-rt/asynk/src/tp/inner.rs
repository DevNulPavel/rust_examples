use super::{queue::JobQueue, JoinError};
use drop_panic::drop_panic;
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle, ThreadId},
};

pub(super) struct Inner {
    name: String,
    job_queue: JobQueue,
    threads: Mutex<Option<HashMap<ThreadId, JoinHandle<()>>>>,
}

impl Inner {
    pub fn new(name: String, thread_count: usize) -> Arc<Self> {
        let this = Arc::new(Self {
            name,
            job_queue: JobQueue::new(),
            threads: Mutex::new(None),
        });

        for _ in 0..thread_count {
            Arc::clone(&this).create_worker();
        }

        this
    }

    /// Enqueue job
    pub fn spawn(&self, job: impl Fn() + Send + 'static) {
        self.job_queue.add(Box::new(job));
    }

    pub fn join(&self) -> Result<(), JoinError> {
        // Notify queue about finish and ask it to wake sleeping threads
        self.job_queue.finish_ntf();

        let mut panicked = 0;

        let Some(threads) = self.threads.lock().take() else {
            return Ok(());
        };

        threads.into_values().for_each(|jh| {
            jh.join().inspect_err(|_| panicked += 1).ok();
        });

        if panicked == 0 {
            Ok(())
        } else {
            Err(JoinError(panicked))
        }
    }

    /// Создать поток для выполнения задач
    fn create_worker(self: Arc<Self>) {
        let this = Arc::clone(&self);

        let jh = thread::Builder::new()
            .name(self.name.clone())
            .spawn(move || {
                // In case of thread panic that object will call the recovery function
                drop_panic! {
                    Arc::clone(&this).panic_handler()
                };

                // Start working cycle
                Arc::clone(&this).worker_routine()
            })
            .expect("spawn worker thread");

        let id = jh.thread().id();

        let mut lock = self.threads.lock();
        match *lock {
            Some(ref mut threads) => match threads.get_mut(&id) {
                Some(_) => panic!("thread with id {:?} already created", id),
                None => {
                    threads.insert(id, jh);
                }
            },
            None => *lock = Some(HashMap::from([(id, jh)])),
        }
    }

    /// Working thread panic handling
    fn panic_handler(self: Arc<Self>) {
        let id = thread::current().id();

        if let Some(threads) = self.threads.lock().as_mut() {
            threads.remove(&id);
        };

        // Recreate the worker thread
        self.create_worker();
    }

    /// Thread working cycle
    fn worker_routine(&self) {
        loop {
            let Some(job) = self.job_queue.get_blocked() else {
                return;
            };

            job();
        }
    }
}
