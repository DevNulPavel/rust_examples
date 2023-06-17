use crate::pool::{BuildError, Counters, SpawnError, Stats};
use crossbeam::utils::Backoff;
use std::{
    sync::{atomic, Arc},
    thread,
    time::Instant,
};

/////////////////////////////////////////////////////////////////////////////////////////

/// Корзина
struct Bucket<J> {
    /// Слот
    slot: BucketSlot<J>,

    /// Тег
    touch_tag: TouchTag,
}

impl<J> Default for Bucket<J> {
    fn default() -> Self {
        Self {
            // Слот с пустой очередью
            slot: BucketSlot {
                jobs_queue: crossbeam::queue::SegQueue::new(),
            },
            // Дефолтный тег
            touch_tag: TouchTag::default(),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Слот, содержащий внутри себя очередь задач
struct BucketSlot<J> {
    jobs_queue: crossbeam::queue::SegQueue<J>,
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Декодированный тег
#[derive(Debug)]
struct TouchTagDecoded {
    /// Когда был взят
    taken_by: usize,

    /// Количество работ
    jobs_count: usize,
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Тег, содержащий внутри атомарную переменную
struct TouchTag {
    tag: atomic::AtomicU64,
}

impl Default for TouchTag {
    fn default() -> TouchTag {
        TouchTag {
            tag: atomic::AtomicU64::new(0),
        }
    }
}

impl TouchTag {
    // Маски, позволяющие упаковать два u32 значения в u64
    const JOBS_COUNT_MASK: u64 = u32::MAX as u64;
    const TAKEN_BY_MASK: u64 = !Self::JOBS_COUNT_MASK;

    /// Получаем из тега его значение
    fn load(&self) -> u64 {
        self.tag.load(atomic::Ordering::Relaxed)
    }

    /// Пробуем заменить старое значение на новое,
    /// выдает ошибку с новым прошлым значением.
    fn try_set(&self, prev_tag: u64, new_tag: u64) -> Result<(), u64> {
        // Заменяем значение в Atomic только если оно совпадает с прошлым, иначе
        // возвращаем ошибку с новым прошлым значением, которое не совпало с текущим.
        // Если прошлое значение у нас совпало с текущим, тогда используется первый
        // параметр порядка.
        // Если сравнение не прокатило, тогда используется второй парамтр.
        //
        // Но что-то не очень понятен смысл второго параметра,
        // если первый влияет на подгрузку + на запись при успешном сравнении.
        // То что делает второй - толком не ясно.
        // Второй параметр может быть: SeqCst, Acquire or Relaxed.
        self.tag
            .compare_exchange_weak(
                prev_tag,
                new_tag,
                atomic::Ordering::Acquire,
                atomic::Ordering::Relaxed,
            )
            .map(|_| ())
    }

    /// Разделяе u64 значение на куски по маске
    fn decompose(tag: u64) -> TouchTagDecoded {
        TouchTagDecoded {
            taken_by: ((tag & Self::TAKEN_BY_MASK) >> 32) as usize,
            jobs_count: (tag & Self::JOBS_COUNT_MASK) as usize,
        }
    }

    /// Собираем в кучу значения
    fn compose(decoded: TouchTagDecoded) -> u64 {
        let mut tag = (decoded.taken_by as u64) << 32;
        tag |= decoded.jobs_count as u64;
        tag
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub struct Inner<J> {
    /// Куча корзин
    buckets: Vec<Bucket<J>>,

    /// Счетчик спавнов
    spawn_index_counter: atomic::AtomicUsize,

    /// Счетчик ожиданий
    await_index_counter: atomic::AtomicUsize,

    /// Флаг завершения
    is_terminated: atomic::AtomicBool,

    /// Счетчики различные для метрик
    counters: Arc<Counters>,
}

impl<J> Inner<J> {
    /// Создаем систему с нужным количеством воркеров и счетчиками
    pub(super) fn new(workers_count: usize, counters: Arc<Counters>) -> Result<Self, BuildError> {
        let buckets = (0..workers_count).map(|_| Bucket::default()).collect();

        Ok(Self {
            buckets,
            spawn_index_counter: atomic::AtomicUsize::new(0),
            await_index_counter: atomic::AtomicUsize::new(0),
            is_terminated: atomic::AtomicBool::new(false),
            counters,
        })
    }

    /// Принудительная простановка флага завершения + будим переданные потоки
    pub(super) fn force_terminate(&self, threads: &[thread::Thread]) {
        // Проставляем флаг завершения работы синхронно
        self.is_terminated.store(true, atomic::Ordering::SeqCst);

        // Принудительно будим каждый поток с помощью его обработчика
        for thread in threads {
            thread.unpark();
        }
    }

    /// Запускаем определенную работу
    pub(super) fn spawn(&self, job: J, threads: &[thread::Thread]) -> Result<(), SpawnError> {
        // Увеличиваем на 1 счетчик спавнов + ограничиваем количеством корзин.
        // Тем самым  мы получаем индекс корзины очередной, куда будем класть нашу задачу.
        let bucket_index = self
            .spawn_index_counter
            .fetch_add(1, atomic::Ordering::Relaxed)
            % self.buckets.len();

        // Получаем ссылку на нужную корзину по индексу
        let bucket = &self.buckets[bucket_index];

        // Получаем из корзины ее тег
        let mut prev_tag = bucket.touch_tag.load();

        // Стартуем цикл
        loop {
            // Не была ли завершена работа еще?
            if self.is_terminated() {
                return Err(SpawnError::ThreadPoolGone);
            }

            // Парсим прошлый тег корзины
            let decoded = TouchTag::decompose(prev_tag);

            // Создаем новый тег корзины в котором делаем +1 количеству запущенных работ
            let new_tag = TouchTag::compose(TouchTagDecoded {
                taken_by: 0,
                jobs_count: decoded.jobs_count + 1,
            });

            // Пробуем для данной корзины обновить тег.
            // Если больше никто не пытался положить в эту корзину, тогда
            // происходит успешная замена прошлого тега на новый.
            if let Err(changed_tag) = bucket.touch_tag.try_set(prev_tag, new_tag) {
                // Если же кто-то другой попытался положить в корзину задачу.
                // Тогда мы обновляем прошлый тег на новое значение.
                prev_tag = changed_tag;

                // Увеличиваем счетчик коллизий
                self.counters
                    .spawn_touch_tag_collisions
                    .fetch_add(1, atomic::Ordering::Relaxed);

                // Пытаемся снова закинуть задачу в корзину
                continue;
            }

            // TODO: Замена тега и добавления в очередь не атомарны, это норм тут?

            // Раз мы смогли успешно заменить тег, то значит можем добавить задачу в очередь?
            bucket.slot.jobs_queue.push(job);

            // Если прошлое значение тега имело индекс потока, который его взял больше 0,
            if decoded.taken_by > 0 {
                // Тогда находим индекс этого потока
                let worker_index = decoded.taken_by - 1;
                // Затем мы будим этот поток, если он спит
                threads[worker_index].unpark();
            }

            break;
        }

        // +1 к счетчику запущенных потоков
        self.counters
            .spawn_total_count
            .fetch_add(1, atomic::Ordering::Relaxed);

        Ok(())
    }

    /// Пытаемся заполучить какую-то задачу очередную
    pub(super) fn acquire_job(&self, worker_index: usize, stats: &mut Stats) -> Option<J> {
        // Текущее время
        let now = Instant::now();

        // Пробуем заполучить
        let maybe_job = self.actually_acquire_job(worker_index, stats);

        // Записываем статистику времени получения для метрик
        stats.acquire_job_time += now.elapsed();
        stats.acquire_job_count += 1;

        maybe_job
    }

    fn actually_acquire_job(&self, worker_index: usize, stats: &mut Stats) -> Option<J> {
        'pick_bucket: loop {
            // Находим индекс нужной нам корзины, увеличивая на 1 счетчик ждущих
            // и вычисляя остаток от деления на количество корзин.
            let bucket_index = self
                .await_index_counter
                .fetch_add(1, atomic::Ordering::Relaxed)
                % self.buckets.len();

            // Ссылка на корзину
            let bucket = &self.buckets[bucket_index];

            let backoff = Backoff::new();
            let mut prev_tag = bucket.touch_tag.load();
            loop {
                if self.is_terminated() {
                    return None;
                }

                let decoded = TouchTag::decompose(prev_tag);

                if decoded.jobs_count == 0 {
                    // empty bucket encountered, have to wait for a job to appear

                    let now = Instant::now();
                    if !backoff.is_completed() {
                        // spin a little
                        backoff.snooze();
                        stats.acquire_job_backoff_time += now.elapsed();
                        stats.acquire_job_backoff_count += 1;
                        prev_tag = bucket.touch_tag.load();
                        continue;
                    }

                    // park the worker if there is no job for a long time
                    if decoded.taken_by != worker_index + 1 {
                        if decoded.taken_by == 0 {
                            // try to acquire parking lot on this bucket
                            let new_tag = TouchTag::compose(TouchTagDecoded {
                                taken_by: worker_index + 1,
                                jobs_count: 0,
                            });
                            if let Err(changed_tag) = bucket.touch_tag.try_set(prev_tag, new_tag) {
                                prev_tag = changed_tag;
                                backoff.reset();
                                continue;
                            }
                        } else {
                            // there is another thread parked on this bucket, proceed to the next one
                            stats.acquire_job_taken_by_collisions += 1;
                            continue 'pick_bucket;
                        }
                    }

                    thread::park();
                    stats.acquire_job_thread_park_time += now.elapsed();
                    stats.acquire_job_thread_park_count += 1;
                    prev_tag = bucket.touch_tag.load();
                    backoff.reset();
                    continue;
                }

                // non-empty bucket, try to reserve a job
                let new_tag = TouchTag::compose(TouchTagDecoded {
                    taken_by: 0,
                    jobs_count: decoded.jobs_count - 1,
                });
                if let Err(changed_tag) = bucket.touch_tag.try_set(prev_tag, new_tag) {
                    prev_tag = changed_tag;
                    continue;
                }

                break;
            }

            // try to pop a job
            let now = Instant::now();
            backoff.reset();
            loop {
                if let Some(job) = bucket.slot.jobs_queue.pop() {
                    stats.acquire_job_seg_queue_pop_time += now.elapsed();
                    stats.acquire_job_seg_queue_pop_count += 1;
                    return Some(job);
                }
                backoff.snooze();
            }
        }
    }

    pub(super) fn is_terminated(&self) -> bool {
        self.is_terminated.load(atomic::Ordering::Relaxed)
    }
}
