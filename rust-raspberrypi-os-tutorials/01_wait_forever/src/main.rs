// Данный код разделен на различные модули, каждый представляет из себя типичную подсистему
// ядра операционной системы.
// Высокоуровневый модуль находится напрямую в src папке. Для примера - src/memory.rs состоит из
// кода, который относится к работе с памятью.

// Архитуктура
//
// Некоторые из подсистем ядра зависит от низкоуровневого кода, который специфичен для конкретной
// архитектуры процессора. Для каждой поддерживаемой архитектуры, там существует подпапка, например,
// `src/_arch/aarch64`
//
// Папка архитектуры отражает модули подсистемы, размещенные в папке src
// Например, код архитектуры, который принадлежит подсистеме памяти - например для src/memory.rs
// путь будет src/_arch/aarch64/memory.rs
// Последний файл напрямую включается и реэкспортруется в src/memory.rs, поэтому архитектурная часть
// является прозрачной. Это значит, что публичная функция foo(), объявленная в
// src/_arch/aarch64/memory.rs, будет доступная как crate::memory::foo().
//
// _ в _arch означает что данная папка не часть стандартной иерархии модулей.
// Содержимое данного файла условно добавляется в соответствующие файлы испольузя
// #[path = "_arch/xxx/yyy.rs"] аттрибуты

// BSP код
// BSP означает Board Support Package. BSP код организован в src/bsp.rs и состоит специфичные для
// платы функции. BSP модуль пытается отразить ядро

// Интерфейсы ядра
//
// Обе папки arch и bsp содержат код, который условно компилируется в зависимости от актуального
// раргета и платы.

//! ## Kernel interfaces
//!
//! Both `arch` and `bsp` contain code that is conditionally compiled depending on the actual target
//! and board for which the kernel is compiled. For example, the `interrupt controller` hardware of
//! the `Raspberry Pi 3` and the `Raspberry Pi 4` is different, but we want the rest of the `kernel`
//! code to play nicely with any of the two without much hassle.
//!
//! In order to provide a clean abstraction between `arch`, `bsp` and `generic kernel code`,
//! `interface` traits are provided *whenever possible* and *where it makes sense*. They are defined
//! in the respective subsystem module and help to enforce the idiom of *program to an interface,
//! not an implementation*. For example, there will be a common IRQ handling interface which the two
//! different interrupt controller `drivers` of both Raspberrys will implement, and only export the
//! interface to the rest of the `kernel`.
//!
//! ```
//!         +-------------------+
//!         | Interface (Trait) |
//!         |                   |
//!         +--+-------------+--+
//!            ^             ^
//!            |             |
//!            |             |
//! +----------+--+       +--+----------+
//! | kernel code |       |  bsp code   |
//! |             |       |  arch code  |
//! +-------------+       +-------------+
//! ```
//!
//! # Summary
//!
//! For a logical `kernel` subsystem, corresponding code can be distributed over several physical
//! locations. Here is an example for the **memory** subsystem:
//!
//! - `src/memory.rs` and `src/memory/**/*`
//!   - Common code that is agnostic of target processor architecture and `BSP` characteristics.
//!     - Example: A function to zero a chunk of memory.
//!   - Interfaces for the memory subsystem that are implemented by `arch` or `BSP` code.
//!     - Example: An `MMU` interface that defines `MMU` function prototypes.
//! - `src/bsp/__board_name__/memory.rs` and `src/bsp/__board_name__/memory/**/*`
//!   - `BSP` specific code.
//!   - Example: The board's memory map (physical addresses of DRAM and MMIO devices).
//! - `src/_arch/__arch_name__/memory.rs` and `src/_arch/__arch_name__/memory/**/*`
//!   - Processor architecture specific code.
//!   - Example: Implementation of the `MMU` interface for the `__arch_name__` processor
//!     architecture.
//!
//! From a namespace perspective, **memory** subsystem code lives in:
//!
//! - `crate::memory::*`
//! - `crate::bsp::memory::*`

// Rust embedded logo for `make doc`.
#![doc(html_logo_url = "https://git.io/JeGIp")]
#![feature(asm)]
#![feature(global_asm)]
#![no_main]
#![no_std]

// Модуль mod cpu предоставляет _start() функцию старта
mod bsp;
mod cpu;
mod panic_wait;
