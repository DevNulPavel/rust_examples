//! Simple, CPU cache-friendly epoch-based reclamation (EBR).
//!
//! ```rust
//! use ebr::Ebr;
//!
//! let ebr: Ebr<Box<u64>> = Ebr::default();
//!
//! let mut guard = ebr.pin();
//!
//! guard.defer_drop(Box::new(1));
//! ```

use std::{
    cell::RefCell,
    collections::VecDeque,
    mem::{take, MaybeUninit},
    num::NonZeroU64,
    sync::{
        atomic::{
            fence, AtomicU64,
            Ordering::{Acquire, Release},
        },
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use shared_local_state::SharedLocalState;

const BUMP_EPOCH_OPS: u64 = 128;
const BUMP_EPOCH_TRAILING_ZEROS: u32 = BUMP_EPOCH_OPS.trailing_zeros();

fn _test_impls() {
    fn send<T: Send>() {}

    send::<Ebr<()>>();
    send::<Inner<(), 128>>();
}

/// Гарбадж коллектор основанный на эпохах с экстремально эффективными операциями в одном потоке.
///
/// Шаблонная переменная слотов описывает на сколько большими могут быть корзины перед тем
/// как пометить их текущим монотонным таймстампом и разместить в очереди на удаление.
///
/// Если `ebr` уничтожен во время того, как есть неутилизированный мусор, то он отправляем
/// текущий мусор в еще один поток. Тогда текущий мусор отправляется в другой поток, чтобы он уничтожил данные.
///
/// Данные должны быть Send и содержать лишь статические ссылки.
#[derive(Debug)]
pub struct Ebr<T: Send + 'static, const SLOTS: usize = 128> {
    inner: RefCell<Inner<T, SLOTS>>,
}

impl<T: Send + 'static, const SLOTS: usize> Ebr<T, SLOTS> {
    pub fn pin(&self) -> Guard<'_, T, SLOTS> {
        // Получаем мутабельный guard на refcell
        let mut inner = self.inner.borrow_mut();
        // Увеличиваем счетчик пинирований
        inner.pins += 1;

        // Получаем счетчик эпох
        let global_current_epoch = inner.global_current_epoch.load(Acquire);
        // Подифицируем хранилище
        inner
            .local_progress_registry
            .access_without_notification(|lqe| lqe.store(global_current_epoch, Release));

        // Сколько битов в конце остается
        let should_bump_epoch = inner.pins.trailing_zeros() == BUMP_EPOCH_TRAILING_ZEROS;
        if should_bump_epoch {
            inner.maintenance();
        }

        Guard { ebr: &self.inner }
    }
}

impl<T: Send + 'static, const SLOTS: usize> Clone for Ebr<T, SLOTS> {
    fn clone(&self) -> Self {
        Ebr {
            inner: RefCell::new(self.inner.borrow().clone()),
        }
    }
}

impl<T: Send + 'static, const SLOTS: usize> Default for Ebr<T, SLOTS> {
    fn default() -> Ebr<T, SLOTS> {
        Ebr {
            inner: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Inner<T: Send + 'static, const SLOTS: usize> {
    // Общие неподвижные эпохи
    local_progress_registry: SharedLocalState<AtomicU64>,

    // Наибольшая эпоха, которую безопасно использовать?
    global_minimum_epoch: Arc<AtomicU64>,

    // Текущее значение эпохи
    global_current_epoch: Arc<AtomicU64>,

    // Очередь эпох с мусором ожидающих удаления
    garbage_queue: VecDeque<Bag<T, SLOTS>>,

    // Текущая корзина, куда мы складываем мусор для удаления
    current_garbage_bag: Bag<T, SLOTS>,

    // Ресивер для мусора из завершенных потоков
    maintenance_lock: Arc<Mutex<Receiver<Bag<T, SLOTS>>>>,

    // Отправитель мусора другим потокам на уничтожение
    orphan_tx: Sender<Bag<T, SLOTS>>,

    // Сколько уже было питонов
    pins: u64,
}

impl<T: Send + 'static, const SLOTS: usize> Drop for Inner<T, SLOTS> {
    fn drop(&mut self) {
        // При вызове drop отправляем весь оставшийся мусор очереди в очередь удаления другим потоком
        for old_bag in take(&mut self.garbage_queue) {
            self.orphan_tx.send(old_bag).unwrap();
        }

        // Есть ли мусор в текущей корзине?
        if self.current_garbage_bag.len > 0 {
            let mut full_bag = take(&mut self.current_garbage_bag);
            // Сохраняем значение эпохи
            full_bag.seal(self.global_current_epoch.load(Acquire));
            self.orphan_tx.send(full_bag).unwrap();
        }
    }
}

impl<T: Send + 'static, const SLOTS: usize> Default for Inner<T, SLOTS> {
    fn default() -> Inner<T, SLOTS> {
        let current_epoch = 1;
        let quiescent_epoch = current_epoch - 1;

        let local_quiescent_epoch = AtomicU64::new(quiescent_epoch);
        let local_progress_registry = SharedLocalState::new(local_quiescent_epoch);

        let (orphan_tx, orphan_rx) = channel();

        Inner {
            local_progress_registry,
            global_current_epoch: Arc::new(AtomicU64::new(current_epoch)),
            global_minimum_epoch: Arc::new(AtomicU64::new(quiescent_epoch)),
            garbage_queue: Default::default(),
            current_garbage_bag: Bag::default(),
            maintenance_lock: Arc::new(Mutex::new(orphan_rx)),
            orphan_tx,
            pins: 0,
        }
    }
}

impl<T: Send + 'static, const SLOTS: usize> Clone for Inner<T, SLOTS> {
    fn clone(&self) -> Inner<T, SLOTS> {
        let global_current_epoch = self.global_current_epoch.load(Acquire);

        let local_quiescent_epoch = AtomicU64::new(global_current_epoch);

        let local_progress_registry = self.local_progress_registry.insert(local_quiescent_epoch);

        Inner {
            local_progress_registry,
            global_minimum_epoch: self.global_minimum_epoch.clone(),
            global_current_epoch: self.global_current_epoch.clone(),
            garbage_queue: Default::default(),
            current_garbage_bag: Bag::default(),
            maintenance_lock: self.maintenance_lock.clone(),
            orphan_tx: self.orphan_tx.clone(),
            pins: 0,
        }
    }
}

impl<T: Send + 'static, const SLOTS: usize> Inner<T, SLOTS> {
    #[cold]
    fn maintenance(&mut self) {
        // Получаем значение эпохи текущей
        let last_epoch = self.global_current_epoch.fetch_add(1, Release);

        // Получаем, если можем, блокировку работы в виде ресивера
        let orphan_rx = if let Ok(orphan_rx) = self.maintenance_lock.try_lock() {
            orphan_rx
        } else {
            return;
        };

        // Раз мы выбрали случайным образом поток, который отвечает за уничтожение данных.
        // Он будет делать следующее:
        // - очистку мусора из очереди потоков, которые уничтожены

        // we have now been "elected" global maintainer,
        // which has responsibility for:
        // * bumping the global quiescent epoch
        // * clearing the orphan garbage queue

        let global_minimum_epoch = self
            .local_progress_registry
            .fold(last_epoch, |min, this| min.min(this.load(Acquire)));

        // Сбрасываем кеши atomic операций, чтобы все потоки видели правильное состояние с этого момента
        fence(Release);

        assert_ne!(global_minimum_epoch, u64::MAX);

        self.global_minimum_epoch
            .fetch_max(global_minimum_epoch, Release);

        // Получаем мусорную корзину из очереди
        while let Ok(bag) = orphan_rx.try_recv() {
            // Если это старые корзины - уничтожаем, иначе добавляем в текущую очередь мусора
            if bag.final_epoch.unwrap().get() < global_minimum_epoch {
                drop(bag)
            } else {
                self.garbage_queue.push_back(bag);
            }
        }
    }
}

pub struct Guard<'a, T: Send + 'static, const SLOTS: usize> {
    ebr: &'a RefCell<Inner<T, SLOTS>>,
}

impl<'a, T: Send + 'static, const SLOTS: usize> Drop for Guard<'a, T, SLOTS> {
    fn drop(&mut self) {
        // set this to a large number to ensure it is not counted by `min_epoch()`
        let inner = self.ebr.borrow();
        inner
            .local_progress_registry
            .access_without_notification(|lqe| lqe.store(u64::MAX, Release));
    }
}

impl<'a, T: Send + 'static, const SLOTS: usize> Guard<'a, T, SLOTS> {
    pub fn defer_drop(&mut self, item: T) {
        let mut ebr = self.ebr.borrow_mut();
        ebr.current_garbage_bag.push(item);

        if ebr.current_garbage_bag.is_full() {
            let mut full_bag = take(&mut ebr.current_garbage_bag);
            let global_current_epoch = ebr.global_current_epoch.load(Acquire);
            full_bag.seal(global_current_epoch);
            ebr.garbage_queue.push_back(full_bag);

            let quiescent = ebr.global_minimum_epoch.load(Acquire).saturating_sub(1);

            assert!(
                global_current_epoch > quiescent,
                "expected global_current_epoch \
                {} to be > quiescent {}",
                global_current_epoch,
                quiescent
            );

            while ebr
                .garbage_queue
                .front()
                .unwrap()
                .final_epoch
                .unwrap()
                .get()
                < quiescent
            {
                let bag = ebr.garbage_queue.pop_front().unwrap();
                drop(bag);
            }
        }
    }
}

#[derive(Debug)]
struct Bag<T, const SLOTS: usize> {
    garbage: [MaybeUninit<T>; SLOTS],
    final_epoch: Option<NonZeroU64>,
    len: usize,
}

impl<T, const SLOTS: usize> Drop for Bag<T, SLOTS> {
    fn drop(&mut self) {
        for index in 0..self.len {
            unsafe {
                self.garbage[index].as_mut_ptr().drop_in_place();
            }
        }
    }
}

impl<T, const SLOTS: usize> Bag<T, SLOTS> {
    fn push(&mut self, item: T) {
        debug_assert!(self.len < SLOTS);
        unsafe {
            self.garbage[self.len].as_mut_ptr().write(item);
        }
        self.len += 1;
    }

    const fn is_full(&self) -> bool {
        self.len == SLOTS
    }

    fn seal(&mut self, epoch: u64) {
        self.final_epoch = Some(NonZeroU64::new(epoch).unwrap());
    }
}

impl<T: Send + 'static, const SLOTS: usize> Default for Bag<T, SLOTS> {
    fn default() -> Bag<T, SLOTS> {
        Bag {
            final_epoch: None,
            len: 0,
            garbage: unsafe { MaybeUninit::<[MaybeUninit<T>; SLOTS]>::uninit().assume_init() },
        }
    }
}
