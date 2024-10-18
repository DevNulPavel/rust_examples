// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>

//! Synchronization primitives.

use core::cell::UnsafeCell;

//--------------------------------------------------------------------------------------------------
// Public Definitions
//--------------------------------------------------------------------------------------------------

/// Synchronization interfaces.
pub mod interface {

    /// Any object implementing this trait guarantees exclusive access to the data contained within
    /// the Mutex for the duration of the provided closure.
    ///
    /// The trait follows the [Rust embedded WG's proposal] and therefore provides some goodness
    /// such as [deadlock prevention].
    ///
    /// # Example
    ///
    /// Since the lock function takes an `&mut self` to enable deadlock-prevention, the trait is
    /// best implemented **for a reference to a container struct**, and has a usage pattern that
    /// might feel strange at first:
    ///
    /// [Rust embedded WG's proposal]: https://github.com/rust-embedded/wg/blob/master/rfcs/0377-mutex-trait.md
    /// [deadlock prevention]: https://github.com/rust-embedded/wg/blob/master/rfcs/0377-mutex-trait.md#design-decisions-and-compatibility
    ///
    /// ```
    /// static MUT: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(0));
    ///
    /// fn foo() {
    ///     let mut r = &MUT; // Note that r is mutable
    ///     r.lock(|data| *data += 1);
    /// }
    /// ```
    pub trait Mutex {
        /// The type of encapsulated data.
        type Data;

        /// Creates a critical section and grants temporary mutable access to the encapsulated data.
        fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
    }

    /// A reader-writer exclusion type.
    ///
    /// The implementing object allows either a number of readers or at most one writer at any point
    /// in time.
    pub trait ReadWriteEx {
        /// The type of encapsulated data.
        type Data;

        /// Grants temporary mutable access to the encapsulated data.
        fn write<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R;

        /// Grants temporary immutable access to the encapsulated data.
        fn read<R>(&mut self, f: impl FnOnce(&Self::Data) -> R) -> R;
    }
}

/// A pseudo-lock for teaching purposes.
///
/// Used to introduce [interior mutability].
///
/// In contrast to a real Mutex implementation, does not protect against concurrent access from
/// other cores to the contained data. This part is preserved for later lessons.
///
/// The lock will only be used as long as it is safe to do so, i.e. as long as the kernel is
/// executing on a single core.
///
/// [interior mutability]: https://doc.rust-lang.org/std/cell/index.html
pub struct IRQSafeNullLock<T: ?Sized> {
    data: UnsafeCell<T>,
}

/// A pseudo-lock that is RW during the single-core kernel init phase and RO afterwards.
///
/// Intended to encapsulate data that is populated during kernel init when no concurrency exists.
pub struct InitStateLock<T: ?Sized> {
    data: UnsafeCell<T>,
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

unsafe impl<T: ?Sized> Sync for IRQSafeNullLock<T> {}

impl<T> IRQSafeNullLock<T> {
    /// Wraps `data` into a new `IRQSafeNullLock`.
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

unsafe impl<T: ?Sized> Sync for InitStateLock<T> {}

impl<T> InitStateLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}

//------------------------------------------------------------------------------
// OS Interface Code
//------------------------------------------------------------------------------
use crate::{exception, state};

impl<T> interface::Mutex for &IRQSafeNullLock<T> {
    type Data = T;

    fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        // In a real lock, there would be code encapsulating this line that ensures that this
        // mutable reference will ever only be given out once at a time.
        let data = unsafe { &mut *self.data.get() };

        // Execute the closure while IRQs are masked.
        exception::asynchronous::exec_with_irq_masked(|| f(data))
    }
}

impl<T> interface::ReadWriteEx for &InitStateLock<T> {
    type Data = T;

    fn write<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
        assert!(
            state::state_manager().state() == state::State::Init,
            "InitStateLock::write called after kernel init phase"
        );
        assert!(
            !exception::asynchronous::is_local_irq_masked(),
            "InitStateLock::write called with IRQs unmasked"
        );

        let data = unsafe { &mut *self.data.get() };

        f(data)
    }

    fn read<R>(&mut self, f: impl FnOnce(&Self::Data) -> R) -> R {
        let data = unsafe { &*self.data.get() };

        f(data)
    }
}
