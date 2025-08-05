use core::sync::atomic::{AtomicU32, Ordering};
use parking_lot_core::{DEFAULT_PARK_TOKEN, DEFAULT_UNPARK_TOKEN, SpinWait, park, unpark_one};

////////////////////////////////////////////////////////////////////////////////

/// Статус взятия семафора
const AVAILABLE: u32 = 1;

/// Статус отпущенности семафора
const TAKEN: u32 = 0;

////////////////////////////////////////////////////////////////////////////////

/// Непосредственно сам семафор
pub(super) struct BinarySemaphore {
    /// Текущее состояние семафора
    state: AtomicU32,
}

impl BinarySemaphore {
    /// Попытка захватить семафор с ожиданием, если кто-то уже другой его взял
    pub(super) fn acquire(&self) {
        // Создаем спинлок для короткого ожидания, чтобы не сразу залезать в ядро.
        // TODO: Здесь используется экспоненциальный прирост ожидания на ожидании spin?
        // Но почему-то его нельзя настроить? По-умолчанию вроде бы как 10 спинов нам доступно.
        let mut spin = SpinWait::new();

        loop {
            // Пробуем установить новое значение для атомарной переменной
            if self
                .state
                .compare_exchange(AVAILABLE, TAKEN, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return;
            }

            // Если не вышло установить блокировку, то попробуем покрутиться
            // в спинлоке, отдавая ядро другим потокам при превышении 3х итераций спинов
            if spin.spin() {
                continue;
            }

            // TODO: Все бы ничего, но вроде бы как при перемещении семафора у нас же поменяется и адрес
            // этой самой переменной, так как нигде нету гарантии пинирования.
            //
            // Для парковки потока нам нужен ключ какой-то.
            // В качестве ключа попробуем использовать адрес в памяти нашей переменной состояния.
            let park_key_usize = {
                // Получаем указатель константный на нашу переменную с состоянием
                let park_key_ptr: *const AtomicU32 = &self.state as *const _;

                // Преобразуем указатель просто в число
                let park_key_usize: usize = park_key_ptr as usize;

                // Выдаем дальше
                park_key_usize
            };

            // Специальная лямбда, которая дополнительно проверит, что блокировка нужна все еще.
            // Вроде бы как она же и помогает от разных случайных пробуждений.
            let park_validate = || self.state.load(Ordering::Relaxed) == TAKEN;

            // Лямбда перед уходом в сон
            let park_before_sleep = || {};

            // Коллбек nаймаута парковки потока
            let park_timed_out = |_park_key, _| {};

            // Значение таймаута
            let park_timeout_val = None;

            // Раз спинлок у нас не прокатил, то нам остается лишь припарковать поток текущий на ожидание
            unsafe {
                park(
                    park_key_usize,
                    park_validate,
                    park_before_sleep,
                    park_timed_out,
                    DEFAULT_PARK_TOKEN,
                    park_timeout_val,
                );
            }

            // Раз поток проснулся, то на новой итерации все равно будем пробовать проверять
            // блокировку с помощью спинлока и текущего состояния
            spin.reset();
        }
    }

    /// Функция снятие блокировки на семафоре
    pub(super) fn release(&self) {
        // Пробуем заменить состояние на разблокированное,
        // если прошлое состояние было состояние блокировки
        if self.state.swap(AVAILABLE, Ordering::Release) == TAKEN {
            // TODO: Все бы ничего, но вроде бы как при перемещении семафора у нас же поменяется и адрес
            // этой самой переменной, так как нигде нету гарантии пинирования.
            //
            // Для парковки потока нам нужен ключ какой-то.
            // В качестве ключа попробуем использовать адрес в памяти нашей переменной состояния.
            let park_key_usize = {
                // Получаем указатель константный на нашу переменную с состоянием
                let park_key_ptr: *const AtomicU32 = &self.state as *const _;

                // Преобразуем указатель просто в число
                let park_key_usize: usize = park_key_ptr as usize;

                // Выдаем дальше
                park_key_usize
            };

            // Функция коллбека перед снятием блокировки,
            // она возвращает токен, который присваивается разблокированной функции
            let unpark_callback_function = |_| DEFAULT_UNPARK_TOKEN;

            // Тогда пробуем пробудить поток по ключу
            unsafe {
                unpark_one(park_key_usize, unpark_callback_function);
            }
        }
    }

    /// Отдельная функция, позволяющая получить guard для автоматического получения
    pub(super) fn lock(&self) -> SemaphoreGuard<'_> {
        self.acquire();
        SemaphoreGuard(self)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(super) struct SemaphoreGuard<'a>(&'a BinarySemaphore);

impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) {
        self.0.release();
    }
}

impl BinarySemaphore {}
