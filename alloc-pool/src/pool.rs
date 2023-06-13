use std::{
    ptr,
    mem::{
        ManuallyDrop,
    },
    sync::{
        Arc,
        atomic::{
            Ordering,
            AtomicBool,
        },
    },
};

use crossbeam_epoch as epoch;

use crate::{
    Inner,
    Unique,
    PoolHead,
};

#[derive(Debug)]
pub struct Pool<T> {
    inner: Arc<PoolHead<T>>,
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Pool<T> {
        Pool { inner: self.inner.clone(), }
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Pool<T> {
    pub fn new() -> Pool<T> {
        Pool {
            inner: Arc::new(PoolHead {
                is_detached: AtomicBool::new(false),
                head: epoch::Atomic::null(),
            }),
        }
    }

    pub fn lend<F>(&self, make_value: F) -> Unique<T> where F: FnOnce() -> T {
        let guard = epoch::pin();
        loop {
            let head = self.inner.head.load(Ordering::Acquire, &guard);
            match unsafe { head.as_ref() } {
                Some(entry) => {
                    let next = entry.next.load(Ordering::Relaxed, &guard);
                    if self.inner.head.compare_exchange(head, next, Ordering::Relaxed, Ordering::Relaxed, &guard).is_ok() {
                        unsafe {
                            guard.defer_destroy(head);
                            let value = ManuallyDrop::into_inner(
                                ptr::read(&entry.value),
                            );
                            return Unique { inner: Inner::new(value, self.inner.clone()), };
                        }
                    }
                },
                None =>
                    return Unique { inner: Inner::new(make_value(), self.inner.clone()), },
            }
        }
    }
}
