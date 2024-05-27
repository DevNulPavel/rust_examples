use std::{
    ptr,
    mem::{
        ManuallyDrop,
    },
    sync::{
        Arc,
        Weak,
        atomic::{
            Ordering,
            AtomicBool,
        },
    },
    ops::{
        Deref,
        DerefMut,
    },
    hash::{
        Hash,
        Hasher,
    },
};

use crossbeam_epoch as epoch;

pub mod pool;
pub mod bytes;

#[derive(Debug)]
pub struct Unique<T> {
    inner: Inner<T>,
}

#[derive(Debug)]
pub struct Shared<T> {
    inner: Arc<Inner<T>>,
}

#[derive(Debug)]
pub struct WeakShared<T> {
    inner: Weak<Inner<T>>,
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Shared<T> {
        Shared { inner: self.inner.clone(), }
    }
}

impl<T> Clone for WeakShared<T> {
    fn clone(&self) -> WeakShared<T> {
        WeakShared { inner: self.inner.clone(), }
    }
}

#[derive(Debug)]
struct Inner<T> {
    value: Option<T>,
    pool_head: Arc<PoolHead<T>>,
}

#[derive(Debug)]
struct PoolHead<T> {
    is_detached: AtomicBool,
    head: epoch::Atomic<Entry<T>>,
}

#[derive(Debug)]
struct Entry<T> {
    value: ManuallyDrop<T>,
    next: epoch::Atomic<Entry<T>>,
}

impl<T> AsRef<T> for Shared<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.inner.value.as_ref().unwrap()
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> PartialEq for Shared<T> where T: PartialEq {
    fn eq(&self, other: &Shared<T>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T> PartialEq<T> for Shared<T> where T: PartialEq {
    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other
    }
}

impl<T> Eq for Shared<T> where T: Eq { }

impl<T> Hash for Shared<T> where T: Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<T> Shared<T> {
    pub fn downgrade(&self) -> WeakShared<T> {
        WeakShared {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

impl<T> WeakShared<T> {
    pub fn upgrade(&self) -> Option<Shared<T>> {
        self.inner.upgrade()
            .map(|arc| Shared { inner: arc, })
    }
}

impl<T> Unique<T> {
    pub fn new_detached(value: T) -> Self {
        Self { inner: Inner::new_detached(value), }
    }

    pub fn freeze(self) -> Shared<T> {
        Shared {
            inner: Arc::new(self.inner),
        }
    }
}

impl<T> Inner<T> {
    fn new(value: T, pool_head: Arc<PoolHead<T>>) -> Inner<T> {
        Inner { value: Some(value), pool_head, }
    }

    fn new_detached(value: T) -> Inner<T> {
        Inner::new(
            value,
            Arc::new(PoolHead {
                is_detached: AtomicBool::new(true),
                head: epoch::Atomic::null(),
            }),
        )
    }
}

impl<T> AsRef<T> for Unique<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.inner.value.as_ref().unwrap()
    }
}

impl<T> Deref for Unique<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> AsMut<T> for Unique<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.inner.value.as_mut().unwrap()
    }
}

impl<T> DerefMut for Unique<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T> PartialEq for Unique<T> where T: PartialEq {
    fn eq(&self, other: &Unique<T>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T> PartialEq<T> for Unique<T> where T: PartialEq {
    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other
    }
}

impl<T> Hash for Unique<T> where T: Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            let mut owned_entry = epoch::Owned::new(Entry {
                value: ManuallyDrop::new(value),
                next: epoch::Atomic::null(),
            });
            let guard = epoch::pin();
            loop {
                if self.pool_head.is_detached.load(Ordering::SeqCst) {
                    // pool is detached, terminate reenqueue process and drop entry
                    let entry_value = &owned_entry.value;
                    let _value = ManuallyDrop::into_inner(
                        unsafe { ptr::read(entry_value) },
                    );
                    break;
                }

                let head = self.pool_head.head.load(Ordering::Relaxed, &guard);
                owned_entry.next.store(head, Ordering::Relaxed);

                match self.pool_head.head.compare_exchange(head, owned_entry, Ordering::Release, Ordering::Relaxed, &guard) {
                    Ok(..) =>
                        break,
                    Err(error) =>
                        owned_entry = error.new,
                }
            }
        }
    }
}

impl<T> Drop for PoolHead<T> {
    fn drop(&mut self) {
        // forbid entries list append
        self.is_detached.store(true, Ordering::SeqCst);

        // drop entries
        let guard = epoch::pin();
        loop {
            let head = self.head.load(Ordering::Acquire, &guard);
            match unsafe { head.as_ref() } {
                Some(entry) => {
                    let next = entry.next.load(Ordering::Relaxed, &guard);
                    if self.head.compare_exchange(head, next, Ordering::Relaxed, Ordering::Relaxed, &guard).is_ok() {
                        unsafe {
                            guard.defer_destroy(head);
                            let _value = ManuallyDrop::into_inner(
                                ptr::read(&entry.value),
                            );
                        }
                    }
                },
                None =>
                    break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        mem::drop,
        sync::{
            Arc,
            atomic::{
                Ordering,
                AtomicUsize,
            },
        },
    };

    use super::{
        pool::Pool,
        bytes::BytesPool,
    };

    #[test]
    fn basic() {
        let mut make_counter = 0;
        let drop_counter = Arc::new(AtomicUsize::new(0));

        #[derive(Debug)]
        struct Sample {
            contents: &'static str,
            drop_counter: Arc<AtomicUsize>,
        }

        impl Drop for Sample {
            fn drop(&mut self) {
                self.drop_counter.fetch_add(1, Ordering::SeqCst);
            }
        }

        let pool = Pool::new();

        let sample_a = "hello, world!";
        let sample_b = "goodbye, world!";

        let value = pool.lend(|| { make_counter += 1; Sample { contents: sample_a, drop_counter: drop_counter.clone(), } });
        assert_eq!(value.contents, sample_a);
        assert_eq!(make_counter, 1);

        drop(value);
        assert_eq!(drop_counter.load(Ordering::SeqCst), 0);

        let value_a = pool.lend(|| { make_counter += 1; Sample { contents: sample_b, drop_counter: drop_counter.clone(), } });
        assert_eq!(value_a.contents, sample_a);
        assert_eq!(make_counter, 1);

        let value_b = pool.lend(|| { make_counter += 1; Sample { contents: sample_b, drop_counter: drop_counter.clone(), } });
        assert_eq!(value_b.contents, sample_b);
        assert_eq!(make_counter, 2);

        let value_a_shared = value_a.freeze();
        assert_eq!(value_a_shared.contents, sample_a);
        let value_a_shared_cloned = value_a_shared.clone();
        assert_eq!(value_a_shared_cloned.contents, sample_a);

        drop(value_a_shared);
        drop(value_a_shared_cloned);
        assert_eq!(drop_counter.load(Ordering::SeqCst), 0);

        let value_a = pool.lend(|| { make_counter += 1; Sample { contents: sample_b, drop_counter: drop_counter.clone(), } });
        assert_eq!(value_a.contents, sample_a);
        assert_eq!(make_counter, 2);

        drop(value_a);
        drop(value_b);
        drop(pool);
        assert_eq!(drop_counter.load(Ordering::SeqCst), make_counter);
    }

    #[test]
    fn bytes_pool_send_sync() {
        let pool = BytesPool::new();
        let bytes = pool.lend();

        std::thread::spawn(move || {
            let _bytes = bytes.freeze();
        });
    }
}
