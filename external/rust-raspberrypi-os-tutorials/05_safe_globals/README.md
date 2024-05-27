# Tutorial 05 - Safe Globals

## tl;dr

A pseudo-lock is introduced; It is a first showcase of OS synchronization primitives and enables
safe access to a global data structure.

## Mutable globals in Rust

When we introduced the globally usable `print!` macros in [tutorial 03], we cheated a bit. Calling
`core::fmt`'s `write_fmt()` function, which takes an `&mut self`, was only working because on each
call, a new instance of `QEMUOutput` was created.

If we would want to preserve some state, e.g. statistics about the number of characters written, we
need to make a single global instance of `QEMUOutput` (in Rust, using the `static` keyword).

A `static QEMU_OUTPUT`, however, would not allow to call functions taking `&mut self`. For that, we
would need a `static mut`, but calling functions that mutate state on `static mut`s is unsafe. The
Rust compiler's reasoning for this is that it can then not prevent anymore that multiple
cores/threads are mutating the data concurrently (it is a global, so everyone can reference it from
anywhere. The borrow checker can't help here).

The solution to this problem is to wrap the global into a synchronization primitive. In our case, a
variant of a *MUTual EXclusion* primitive. `Mutex` is introduced as a trait in `synchronization.rs`,
and implemented by the `NullLock` in the same file. In order to make the code lean for teaching
purposes, it leaves out the actual architecture-specific logic for protection against concurrent
access, since we don't need it as long as the kernel only executes on a single core with interrupts
disabled. That is also why it is implemented in the same file as the interface itself. In later
tutorials, an implementation might move to the `_arch` once it pulls in arch-specific code that can
not be further abstracted.

The `NullLock` focuses on showcasing the Rust core concept of [interior mutability]. Make sure to
read up on it. I also recommend to read this article about an [accurate mental model for Rust's
reference types].

If you want to compare the `NullLock` to some real-world mutex implementations, you can check out
implemntations in the [spin crate] or the [parking lot crate].

[tutorial 03]: ../03_hacky_hello_world
[interior mutability]: https://doc.rust-lang.org/std/cell/index.html
[accurate mental model for Rust's reference types]: https://docs.rs/dtolnay/0.0.6/dtolnay/macro._02__reference_types.html
[spin crate]: https://github.com/mvdnes/spin-rs
[parking lot crate]: https://github.com/Amanieu/parking_lot

## Test it

```console
$ make qemu
[...]
[0] Hello from pure Rust!
[1] Chars written: 27
[2] Stopping here.
```

## Diff to previous
```diff

diff -uNr 04_zero_overhead_abstraction/src/bsp/raspberrypi/console.rs 05_safe_globals/src/bsp/raspberrypi/console.rs
--- 04_zero_overhead_abstraction/src/bsp/raspberrypi/console.rs
+++ 05_safe_globals/src/bsp/raspberrypi/console.rs
@@ -4,7 +4,7 @@

 //! BSP console facilities.

-use crate::console;
+use crate::{console, synchronization, synchronization::NullLock};
 use core::fmt;

 //--------------------------------------------------------------------------------------------------
@@ -12,25 +12,64 @@
 //--------------------------------------------------------------------------------------------------

 /// A mystical, magical device for generating QEMU output out of the void.
-struct QEMUOutput;
+///
+/// The mutex protected part.
+struct QEMUOutputInner {
+    chars_written: usize,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// The main struct.
+pub struct QEMUOutput {
+    inner: NullLock<QEMUOutputInner>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Global instances
+//--------------------------------------------------------------------------------------------------
+
+static QEMU_OUTPUT: QEMUOutput = QEMUOutput::new();

 //--------------------------------------------------------------------------------------------------
 // Private Code
 //--------------------------------------------------------------------------------------------------

+impl QEMUOutputInner {
+    const fn new() -> QEMUOutputInner {
+        QEMUOutputInner { chars_written: 0 }
+    }
+
+    /// Send a character.
+    fn write_char(&mut self, c: char) {
+        unsafe {
+            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+        }
+
+        self.chars_written += 1;
+    }
+}
+
 /// Implementing `core::fmt::Write` enables usage of the `format_args!` macros, which in turn are
 /// used to implement the `kernel`'s `print!` and `println!` macros. By implementing `write_str()`,
 /// we get `write_fmt()` automatically.
 ///
+/// The function takes an `&mut self`, so it must be implemented for the inner struct.
+///
 /// See [`src/print.rs`].
 ///
 /// [`src/print.rs`]: ../../print/index.html
-impl fmt::Write for QEMUOutput {
+impl fmt::Write for QEMUOutputInner {
     fn write_str(&mut self, s: &str) -> fmt::Result {
         for c in s.chars() {
-            unsafe {
-                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
+            // Convert newline to carrige return + newline.
+            if c == '\n' {
+                self.write_char('\r')
             }
+
+            self.write_char(c);
         }

         Ok(())
@@ -41,7 +80,39 @@
 // Public Code
 //--------------------------------------------------------------------------------------------------

+impl QEMUOutput {
+    /// Create a new instance.
+    pub const fn new() -> QEMUOutput {
+        QEMUOutput {
+            inner: NullLock::new(QEMUOutputInner::new()),
+        }
+    }
+}
+
 /// Return a reference to the console.
-pub fn console() -> impl console::interface::Write {
-    QEMUOutput {}
+pub fn console() -> &'static impl console::interface::All {
+    &QEMU_OUTPUT
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+use synchronization::interface::Mutex;
+
+/// Passthrough of `args` to the `core::fmt::Write` implementation, but guarded by a Mutex to
+/// serialize access.
+impl console::interface::Write for QEMUOutput {
+    fn write_fmt(&self, args: core::fmt::Arguments) -> fmt::Result {
+        // Fully qualified syntax for the call to `core::fmt::Write::write:fmt()` to increase
+        // readability.
+        let mut r = &self.inner;
+        r.lock(|inner| fmt::Write::write_fmt(inner, args))
+    }
+}
+
+impl console::interface::Statistics for QEMUOutput {
+    fn chars_written(&self) -> usize {
+        let mut r = &self.inner;
+        r.lock(|inner| inner.chars_written)
+    }
 }

diff -uNr 04_zero_overhead_abstraction/src/console.rs 05_safe_globals/src/console.rs
--- 04_zero_overhead_abstraction/src/console.rs
+++ 05_safe_globals/src/console.rs
@@ -10,10 +10,22 @@

 /// Console interfaces.
 pub mod interface {
+    use core::fmt;
+
     /// Console write functions.
-    ///
-    /// `core::fmt::Write` is exactly what we need for now. Re-export it here because
-    /// implementing `console::Write` gives a better hint to the reader about the
-    /// intention.
-    pub use core::fmt::Write;
+    pub trait Write {
+        /// Write a Rust format string.
+        fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result;
+    }
+
+    /// Console statistics.
+    pub trait Statistics {
+        /// Return the number of characters written.
+        fn chars_written(&self) -> usize {
+            0
+        }
+    }
+
+    /// Trait alias for a full-fledged console.
+    pub trait All = Write + Statistics;
 }

diff -uNr 04_zero_overhead_abstraction/src/main.rs 05_safe_globals/src/main.rs
--- 04_zero_overhead_abstraction/src/main.rs
+++ 05_safe_globals/src/main.rs
@@ -95,6 +95,7 @@
 #![feature(format_args_nl)]
 #![feature(naked_functions)]
 #![feature(panic_info_message)]
+#![feature(trait_alias)]
 #![no_main]
 #![no_std]

@@ -108,6 +109,7 @@
 mod panic_wait;
 mod print;
 mod runtime_init;
+mod synchronization;

 /// Early init code.
 ///
@@ -115,8 +117,15 @@
 ///
 /// - Only a single core must be active and running this function.
 unsafe fn kernel_init() -> ! {
+    use console::interface::Statistics;
+
     println!("[0] Hello from pure Rust!");

-    println!("[1] Stopping here.");
+    println!(
+        "[1] Chars written: {}",
+        bsp::console::console().chars_written()
+    );
+
+    println!("[2] Stopping here.");
     cpu::wait_forever()
 }

diff -uNr 04_zero_overhead_abstraction/src/synchronization.rs 05_safe_globals/src/synchronization.rs
--- 04_zero_overhead_abstraction/src/synchronization.rs
+++ 05_safe_globals/src/synchronization.rs
@@ -0,0 +1,92 @@
+// SPDX-License-Identifier: MIT OR Apache-2.0
+//
+// Copyright (c) 2020 Andre Richter <andre.o.richter@gmail.com>
+
+//! Synchronization primitives.
+
+use core::cell::UnsafeCell;
+
+//--------------------------------------------------------------------------------------------------
+// Public Definitions
+//--------------------------------------------------------------------------------------------------
+
+/// Synchronization interfaces.
+pub mod interface {
+
+    /// Any object implementing this trait guarantees exclusive access to the data contained within
+    /// the Mutex for the duration of the provided closure.
+    ///
+    /// The trait follows the [Rust embedded WG's proposal] and therefore provides some goodness
+    /// such as [deadlock prevention].
+    ///
+    /// # Example
+    ///
+    /// Since the lock function takes an `&mut self` to enable deadlock-prevention, the trait is
+    /// best implemented **for a reference to a container struct**, and has a usage pattern that
+    /// might feel strange at first:
+    ///
+    /// [Rust embedded WG's proposal]: https://github.com/rust-embedded/wg/blob/master/rfcs/0377-mutex-trait.md
+    /// [deadlock prevention]: https://github.com/rust-embedded/wg/blob/master/rfcs/0377-mutex-trait.md#design-decisions-and-compatibility
+    ///
+    /// ```
+    /// static MUT: Mutex<RefCell<i32>> = Mutex::new(RefCell::new(0));
+    ///
+    /// fn foo() {
+    ///     let mut r = &MUT; // Note that r is mutable
+    ///     r.lock(|data| *data += 1);
+    /// }
+    /// ```
+    pub trait Mutex {
+        /// The type of encapsulated data.
+        type Data;
+
+        /// Creates a critical section and grants temporary mutable access to the encapsulated data.
+        fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R;
+    }
+}
+
+/// A pseudo-lock for teaching purposes.
+///
+/// Used to introduce [interior mutability].
+///
+/// In contrast to a real Mutex implementation, does not protect against concurrent access from
+/// other cores to the contained data. This part is preserved for later lessons.
+///
+/// The lock will only be used as long as it is safe to do so, i.e. as long as the kernel is
+/// executing single-threaded, aka only running on a single core with interrupts disabled.
+///
+/// [interior mutability]: https://doc.rust-lang.org/std/cell/index.html
+pub struct NullLock<T: ?Sized> {
+    data: UnsafeCell<T>,
+}
+
+//--------------------------------------------------------------------------------------------------
+// Public Code
+//--------------------------------------------------------------------------------------------------
+
+unsafe impl<T: ?Sized> Sync for NullLock<T> {}
+
+impl<T> NullLock<T> {
+    /// Wraps `data` into a new `NullLock`.
+    pub const fn new(data: T) -> Self {
+        Self {
+            data: UnsafeCell::new(data),
+        }
+    }
+}
+
+//------------------------------------------------------------------------------
+// OS Interface Code
+//------------------------------------------------------------------------------
+
+impl<T> interface::Mutex for &NullLock<T> {
+    type Data = T;
+
+    fn lock<R>(&mut self, f: impl FnOnce(&mut Self::Data) -> R) -> R {
+        // In a real lock, there would be code encapsulating this line that ensures that this
+        // mutable reference will ever only be given out once at a time.
+        let data = unsafe { &mut *self.data.get() };
+
+        f(data)
+    }
+}

```
