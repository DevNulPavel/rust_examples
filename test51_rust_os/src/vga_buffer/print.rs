use core::{
    fmt::{
        self,
        Write
    }
};
use super::{
    writer::{
        WRITER
    }
};

// Благодаря данному аттрибуту, данный макрос будет доступен сразу же от корня крейта,
// импортировать не нужно будет полный путь
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        // $crate - вроде бы значит, что мы используем данный крейт в пространстве имен
        $crate::vga_buffer::print::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Благодаря данному аттрибуту данный метод не будет отображаться в IDE при автокомплите
// https://doc.rust-lang.org/nightly/rustdoc/the-doc-attribute.html#dochidden
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}