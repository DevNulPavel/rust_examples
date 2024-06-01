use super::{pool::JoinError, queue::JobQueue};
use parking_lot::{Mutex, MutexGuard};
use std::{
    collections::{hash_map::Entry, HashMap},
    num::NonZeroUsize,
    sync::Arc,
    thread::{self, JoinHandle, ThreadId},
};

////////////////////////////////////////////////////////////////////////////////

type ThreadsHashmap = HashMap<ThreadId, JoinHandle<()>>;

////////////////////////////////////////////////////////////////////////////////

pub(super) struct Inner {
    /// Очередь задач
    job_queue: JobQueue,

    /// Имя задач в пуле
    name: String,

    /// Текущие запущенные потоки + возможность их ожидания.
    /// Обернуто в Mutex, так как завершать работу у нас может любой поток,
    /// этот поток пусть и будет отвечать за обработку ошибок.
    threads: Mutex<Option<ThreadsHashmap>>,
}

impl Inner {
    /// Создаем внутреннюю обертку пула
    pub(super) fn new_arc(name: String, thread_count: NonZeroUsize) -> Arc<Inner> {
        // Сначала создаем Arc для текущего пула
        let this = Arc::new(Inner {
            job_queue: JobQueue::new(),
            name,
            threads: Mutex::new(Some(ThreadsHashmap::with_capacity(thread_count.get()))),
        });

        {
            // Блокируемся на пуле потоков для добавления нового потока
            let mut lock = this.threads.lock();

            for _ in 0..thread_count.get() {
                create_worker(&this, &mut lock);
            }
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
    pub(super) fn join(&self) -> Result<(), JoinError> {
        // Берем блокировку на все время ожидания,
        // сейчас мы не знаем, будем ли именно мы текущим потоком завершать работу.
        let mut threads_lock = self.threads.lock();

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
}

////////////////////////////////////////////////////////////////////////////////

fn create_worker<'a: 'b, 'b>(
    inner_arc: &'a Arc<Inner>,
    threads_lock: &'b mut MutexGuard<'a, Option<ThreadsHashmap>>,
) {
    // Создаем билдер для потоков
    let jh = thread::Builder::new()
        // Прописываем там нужное имя потока
        .name(inner_arc.name.clone())
        // Запускаем в работу поток
        .spawn({
            // Шарим Arc обработчика для потока
            let inner_for_thread = Arc::clone(inner_arc);

            move || {
                // Если данный поток у нас запаникует, тогда у нас вызовется обработчик данный.
                // drop_panic! {};
                let _guard = drop_panic::guard({
                    let inner_for_thread_panic = Arc::clone(&inner_for_thread);
                    move || {
                        panic_handler(&inner_for_thread_panic);
                    }
                });

                // Запускаем задачу
                worker_routine(&inner_for_thread);
            }
        })
        .expect("spawn worker thread");

    // Получаем идентификатор потока
    let id = jh.thread().id();

    // Смотрим на текущие блокировки
    match threads_lock.as_mut() {
        // У нас уже есть хранилище потоков?
        Some(threads) => {
            // Проверяем наличие такого потока в пуле потоков
            match threads.entry(id) {
                // Такого элемента у нас нету
                Entry::Vacant(location) => {
                    location.insert(jh);
                }
                // Элемент у нас есть почему-то уже такой
                Entry::Occupied(_) => {
                    // Паникуем, какая-то фундаментальная проблема системная
                    panic!("thread with id {:?} already created", id)
                }
            }
        }
        // По какой-то причине нету хешмапы?
        None => {
            // Создадим тогда
            **threads_lock = Some(ThreadsHashmap::from([(id, jh)]));
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Обработчик паники одного из потоков
fn panic_handler(inner: &Arc<Inner>) {
    // У нас есть еще куда добавлять потоки?
    let mut lock = inner.threads.lock();

    // У нас есть еще куда добавлять потоки?
    let Some(threads) = lock.as_mut() else {
        return;
    };

    // Получаем идентификатор запаниковавшего потока
    let id = thread::current().id();

    // Удаляем старый join
    threads.remove(&id);

    // Создаем заново новый поток взамен одного утраченного
    create_worker(inner, &mut lock);
}

////////////////////////////////////////////////////////////////////////////////

/// Цикл обработки задач
fn worker_routine(inner: &Inner) {
    loop {
        let Some(job) = inner.job_queue.get_blocked() else {
            return;
        };

        job();
    }
}
