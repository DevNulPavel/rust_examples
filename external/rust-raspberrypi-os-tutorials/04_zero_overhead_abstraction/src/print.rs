// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Printing facilities.

use crate::{bsp, console};
use core::fmt;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use console::interface::Write;

    bsp::console::console().write_fmt(args).unwrap();
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Prints without a newline.
///
/// Carbon copy from https://doc.rust-lang.org/src/std/macros.rs.html
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

/// Prints with a newline.
///
/// Carbon copy from https://doc.rust-lang.org/src/std/macros.rs.html
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::_print(format_args_nl!($($arg)*));
    })
}
