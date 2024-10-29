use super::job::Job;
use parking_lot::{Condvar, Mutex};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicBool, Ordering},
};

////////////////////////////////////////////////////////////////////////////////

/// Очередь задач многопоточная
#[derive(Default)]
pub(super) struct JobQueue {
    /// Очередь под блокировкой
    queue: Mutex<VecDeque<Job>>,

    /// Condvar для блокировки выше
    not_empty: Condvar,

    /// Флаг завершения работы.
    finished: AtomicBool,
}

impl JobQueue {
    pub(super) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Добавляем новую задачу, будим потоки
    pub(super) fn add(&self, job: Job) {
        // Если был проставлен флаг завершения - выходим
        if self.finished.load(Ordering::Acquire) {
            return;
        }

        // Блокируемся и пушим задачу в очередь
        self.queue.lock().push_back(job);

        // Оповещаем кого-то, что прилетела новая задача.
        // Здесь атомарность у нас не нужна, так как атомарно надо снимать блокировку
        // и переходить в ожидание, чтобы не потерять уведомление.
        self.not_empty.notify_one();
    }

    /* /// Говорим, что нужно завершать работу всех обработчиков задач
    pub(super) fn finish_ntf(&self) {
        // Проставляем флаг завершения
        self.finished.store(true, Ordering::Release);

        // Уведомляем всех проснуться
        self.not_empty.notify_all();
    } */

    /// Извлекаем очередную задачу из очереди задач. Если очередь у нас пустая, тогда
    /// поток у нас будет спать до тех пор, пока задача не прилетит.
    ///
    /// Если очередь закрыта, тогда возвращается None.
    ///
    pub(super) fn get_blocked(&self) -> Option<Job> {
        // Работа завершена?
        // Проверим до блокировки
        if self.finished.load(Ordering::Acquire) {
            return None;
        }

        // Берем блокировку
        let mut lock = self.queue.lock();

        loop {
            // Пробуем извлечь шапку
            match lock.pop_front() {
                // Есть значение
                Some(val) => {
                    // Результат
                    return Some(val);
                }
                // Нет значения пока
                None => {
                    // Мы атомарно снимаем блокировку и ждем уведомлений.
                    // За счет атомарности мы не пропустим точно флаг пробуждения
                    // после снятия блокировки, но до постановки в режим ожидания.
                    self.not_empty.wait(&mut lock);

                    // Если поток был пробужден из-за завершения? Завершаемся тогда
                    if self.finished.load(Ordering::Acquire) {
                        return None;
                    }
                }
            }
        }
    }
}
