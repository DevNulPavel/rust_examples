use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::Mutex;

/// Синхронная очередь
pub struct ParallelQueue<T> {
    queue: Mutex<Option<Sender<T>>>,
}

impl<T> ParallelQueue<T> {
    /// Создаем инстанс на определенное количество ридеров
    ///
    /// # Аргументы
    ///
    /// - `queues`: сколько ридеров
    /// - `capacity`: емкость каждой внутренней очереди
    pub fn new(queues: usize, capacity: usize) -> (Self, Vec<Receiver<T>>) {
        // Массив ресиверов
        let mut receivers = Vec::with_capacity(queues);
        // Создаем канал ограниченного размера
        let (tx, rx) = bounded(capacity);

        // Заполняем массив клонами ресивера
        for _ in 0..queues {
            receivers.push(rx.clone());
        }

        // Создаем очередь
        let queue = ParallelQueue {
            queue: Mutex::new(Some(tx)),
        };

        // Создаем очередь и ресиверы
        (queue, receivers)
    }

    // Отправляем что-то в очередь
    pub fn send(&self, v: T) {
        if let Some(s) = self.queue.lock().unwrap().as_ref() {
            let _ = s.send(v);
        }
    }

    // Закрываем очередь
    pub fn close(&self) {
        *self.queue.lock().unwrap() = None;
    }
}
