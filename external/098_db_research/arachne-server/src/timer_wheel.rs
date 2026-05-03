use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam_queue::SegQueue;

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct TimerEntry {
    pub user_id: u32,
    pub trigger_ts: u64,
    pub version: u32,          // Эпоха, для которой этот таймер валиден
    pub is_lock_timeout: bool, // Если true - значит истек лок. Если false - пришло время следующей проверки
}

#[derive(Debug)]
pub struct HierarchicalWheel {
    seconds: [SegQueue<TimerEntry>; 60],
    minutes: [SegQueue<TimerEntry>; 60],
    hours: [SegQueue<TimerEntry>; 24],
    // Считаем, что в месяце 30 дней
    days: [SegQueue<TimerEntry>; 30],
    // Считаем 12 месяцев (покрывает 360 дней)
    months: [SegQueue<TimerEntry>; 12],
    // Переполнение: все, что отложено более чем на 360 дней
    years: SegQueue<TimerEntry>,
    locked_count: AtomicUsize,
    waiting_count: AtomicUsize,
}

impl Default for HierarchicalWheel {
    fn default() -> Self {
        Self {
            seconds: std::array::from_fn(|_| SegQueue::new()),
            minutes: std::array::from_fn(|_| SegQueue::new()),
            hours: std::array::from_fn(|_| SegQueue::new()),
            days: std::array::from_fn(|_| SegQueue::new()),
            months: std::array::from_fn(|_| SegQueue::new()),
            years: SegQueue::new(),
            locked_count: AtomicUsize::new(0),
            waiting_count: AtomicUsize::new(0),
        }
    }
}

impl HierarchicalWheel {
    /// O(1) Добавление таймера в соответствующий слот иерархии
    pub fn insert(&self, current_ts: u64, entry: TimerEntry) {
        if entry.is_lock_timeout {
            self.locked_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.waiting_count.fetch_add(1, Ordering::Relaxed);
        }

        self.insert_internal(current_ts, entry);
    }

    fn insert_internal(&self, current_ts: u64, entry: TimerEntry) {
        let trigger = std::cmp::max(current_ts, entry.trigger_ts);
        let diff = trigger - current_ts;

        if diff < 60 {
            self.seconds[(trigger % 60) as usize].push(entry);
        } else if diff < 3600 {
            self.minutes[((trigger / 60) % 60) as usize].push(entry);
        } else if diff < 86400 {
            self.hours[((trigger / 3600) % 24) as usize].push(entry);
        } else if diff < 2592000 {
            // 30 суток (месяц)
            self.days[((trigger / 86400) % 30) as usize].push(entry);
        } else if diff < 31104000 {
            // 360 суток (год)
            self.months[((trigger / 2592000) % 12) as usize].push(entry);
        } else {
            // Больше 360 дней
            self.years.push(entry);
        }
    }

    /// Получает готовые к выдаче таймеры для текущей секунды,
    /// выполняя каскадирование таймеров с верхних уровней.
    pub fn tick(&self, current_ts: u64) -> Vec<TimerEntry> {
        let mut ready = Vec::new();

        // Каскадирования срабатывают строго на границах циклов.
        // Задачи спускаются вниз по иерархии в более "быстрые" слоты.

        // 1. Каскадируем переполнение годов (1 раз в 360 дней)
        // Используем фиксированный len() + for, так как элементы могут передобавиться обратно в years
        if current_ts > 0 && current_ts.is_multiple_of(31104000) {
            let len = self.years.len();
            for _ in 0..len {
                if let Some(task) = self.years.pop() {
                    self.insert_internal(current_ts, task);
                }
            }
        }

        // 2. Каскадируем месяцы в дни (1 раз в 30 дней)
        // while let Some безопасен, так как при insert элемент гарантированно попадет в ДРУГОЙ слот
        if current_ts > 0 && current_ts.is_multiple_of(2592000) {
            let month_slot = ((current_ts / 2592000) % 12) as usize;
            while let Some(task) = self.months[month_slot].pop() {
                self.insert_internal(current_ts, task);
            }
        }

        // 3. Каскадируем дни в часы (1 раз в 24 часа)
        if current_ts > 0 && current_ts.is_multiple_of(86400) {
            let day_slot = ((current_ts / 86400) % 30) as usize;
            while let Some(task) = self.days[day_slot].pop() {
                self.insert_internal(current_ts, task);
            }
        }

        // 4. Каскадируем часы в минуты (1 раз в час)
        if current_ts > 0 && current_ts.is_multiple_of(3600) {
            let hour_slot = ((current_ts / 3600) % 24) as usize;
            while let Some(task) = self.hours[hour_slot].pop() {
                self.insert_internal(current_ts, task);
            }
        }

        // 5. Каскадируем минуты в секунды (1 раз в минуту)
        if current_ts > 0 && current_ts.is_multiple_of(60) {
            let min_slot = ((current_ts / 60) % 60) as usize;
            while let Some(task) = self.minutes[min_slot].pop() {
                self.insert_internal(current_ts, task);
            }
        }

        // 6. Выдаем готовые таймеры текущей секунды
        let sec_slot = (current_ts % 60) as usize;
        while let Some(task) = self.seconds[sec_slot].pop() {
            // Уменьшаем нужный счетчик перед выдачей
            if task.is_lock_timeout {
                self.locked_count.fetch_sub(1, Ordering::Relaxed);
            } else {
                self.waiting_count.fetch_sub(1, Ordering::Relaxed);
            }
            ready.push(task);
        }

        ready
    }

    /// Получает общее количество элементов во всем колесе
    pub fn len(&self) -> usize {
        self.locked_len() + self.waiting_len()
    }

    /// Возвращает количество пользователей, заблокированных на время обработки воркером
    pub fn locked_len(&self) -> usize {
        self.locked_count.load(Ordering::Relaxed)
    }

    /// Возвращает количество пользователей, ожидающих своего времени
    pub fn waiting_len(&self) -> usize {
        self.waiting_count.load(Ordering::Relaxed)
    }

    /// Опустошает структуру, переводя в массив (используется для снапшотов БД)
    pub fn drain(&self) -> Vec<TimerEntry> {
        let mut all = Vec::new();
        let mut extract = |q: &SegQueue<TimerEntry>| {
            while let Some(item) = q.pop() {
                all.push(item);
            }
        };

        for q in &self.seconds {
            extract(q);
        }
        for q in &self.minutes {
            extract(q);
        }
        for q in &self.hours {
            extract(q);
        }
        for q in &self.days {
            extract(q);
        }
        for q in &self.months {
            extract(q);
        }
        extract(&self.years);
        self.locked_count.store(0, Ordering::Relaxed);
        self.waiting_count.store(0, Ordering::Relaxed);

        all
    }

    /// Восстанавливает стейт из массива сущностей
    pub fn restore(entries: Vec<TimerEntry>, current_ts: u64) -> Self {
        let wheel = Self::default();
        for entry in entries {
            wheel.insert(current_ts, entry);
        }
        wheel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seconds_wheel() {
        let wheel = HierarchicalWheel::default();
        let current_ts = 1000;

        wheel.insert(
            current_ts,
            TimerEntry {
                user_id: 1,
                trigger_ts: 1005,
                version: 1,
                is_lock_timeout: false,
            },
        );

        assert_eq!(wheel.len(), 1);

        for ts in 1000..1005 {
            assert!(wheel.tick(ts).is_empty());
        }

        let ready = wheel.tick(1005);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].user_id, 1);
        assert_eq!(wheel.len(), 0);
    }

    #[test]
    fn test_minutes_cascade() {
        let wheel = HierarchicalWheel::default();
        let current_ts = 3600;

        // 65 секунд - попадет в минутное кольцо
        wheel.insert(
            current_ts,
            TimerEntry {
                user_id: 2,
                trigger_ts: 3665,
                version: 1,
                is_lock_timeout: false,
            },
        );

        // До 3659 пусто
        for ts in 3600..3660 {
            assert!(wheel.tick(ts).is_empty());
        }

        // На 3660 (граница минуты) происходит каскад из минут в секунды
        assert!(wheel.tick(3660).is_empty());

        // Секунды с 3661 по 3664 тоже пусты (задача ждет своего слота секунд)
        for ts in 3661..3665 {
            assert!(wheel.tick(ts).is_empty());
        }

        // Наконец, на 3665 секунде задача должна выстрелить
        let ready = wheel.tick(3665);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].user_id, 2);
    }

    #[test]
    fn test_drain_and_restore() {
        let wheel = HierarchicalWheel::default();
        let current_ts = 1000;

        wheel.insert(
            current_ts,
            TimerEntry {
                user_id: 10,
                trigger_ts: 1050,
                version: 1,
                is_lock_timeout: false,
            },
        );
        wheel.insert(
            current_ts,
            TimerEntry {
                user_id: 20,
                trigger_ts: 5000,
                version: 1,
                is_lock_timeout: false,
            },
        );
        wheel.insert(
            current_ts,
            TimerEntry {
                user_id: 30,
                trigger_ts: 100000,
                version: 1,
                is_lock_timeout: false,
            },
        );

        assert_eq!(wheel.len(), 3);

        let drained = wheel.drain();
        assert_eq!(drained.len(), 3);
        assert_eq!(wheel.len(), 0);

        let restored = HierarchicalWheel::restore(drained, current_ts);
        assert_eq!(restored.len(), 3);

        for ts in 1000..1050 {
            restored.tick(ts);
        }
        let ready = restored.tick(1050);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].user_id, 10);
    }
}
