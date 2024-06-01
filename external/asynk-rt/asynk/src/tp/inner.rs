use super::{queue::JobQueue, JoinError};
use drop_panic::drop_panic;
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle, ThreadId},
};

////////////////////////////////////////////////////////////////////////////////

pub(super) struct Inner {
    /// Имя задач в пуле
    name: String,

    /// Очередь задач
    job_queue: JobQueue,

    /// Текущие запущенные потоки + возможность их ожидания.
    /// Обернуто в Mutex, так как завершать работу у нас может любой поток,
    /// этот поток пусть и будет отвечать за обработку ошибок.
    threads: Mutex<Option<HashMap<ThreadId, JoinHandle<()>>>>,
}

impl Inner {
    /// Создаем внутреннюю обертку пула
    pub(super) fn new_arc(name: String, thread_count: usize) -> Arc<Self> {
        // Сначала создаем Arc для текущего пула
        let this = Arc::new(Inner {
            name,
            job_queue: JobQueue::new(),
            threads: Mutex::new(None),
        });

        // Затем создаем нужное количество потоков, которые будут шарить данный Arc
        for _ in 0..thread_count {
            let this_clone = Arc::clone(&this);

            this_clone.create_worker();
        }

        this
    }

    /// Новая задача у пуле задач
    pub(super) fn spawn<J>(&self, job: J)
    where
        // Функция разового запуска, которую можем перемещать
        // из одного потока в другой и выполнять.
        // Этот функтор может содержать лишь статические ссылки.
        J: FnOnce() + Send + 'static,
    {
        // Данную задачу оборачиваем в Box, так как задачи
        // бывают у нас совершенно разного размера на стеке
        self.job_queue.add(Box::new(job));
    }

    /// Дожидаемся завершения работы вообще всех задач в пуле.
    pub(super) fn join(self) -> Result<(), JoinError> {
        // Берем блокировку на все время ожидания,
        // сейчас мы не знаем, будем ли именно мы текущим потоком завершать работу.
        let threads_lock = self.threads.lock();

        // Так как вызываться завершение может совершенно в ином потоке,
        // так что проверяем, не завершил ли кто-то еще?
        let Some(threads) = threads_lock.take() else {
            return Ok(());
        };

        // Оповещаем очередь о желании завершения, будим все спящие потоки
        // для завершения.
        self.job_queue.finish_ntf();

        // Сколько задач запаниковало?
        let mut panicked = 0;

        // Ждем завершения каждого потока, отслеживаем сколько запаниковало
        threads.into_values().for_each(|jh| {
            jh.join().inspect_err(|_| panicked += 1).ok();
        });

        // Если здесь хоть один запаниковал, значит ошибка
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
