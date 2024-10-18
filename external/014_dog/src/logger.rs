//! Debug error logging.

use std::ffi::OsStr;
use ansi_term::{Colour, ANSIString};


#[derive(Debug)]
struct Logger{
}

const GLOBAL_LOGGER: &Logger = &Logger;

// Перегоняем уровень логирования в цветной текст
fn level(level: log::Level) -> ANSIString<'static> {
    match level {
        log::Level::Error => Colour::Red.paint("ERROR"),
        log::Level::Warn  => Colour::Yellow.paint("WARN"),
        log::Level::Info  => Colour::Cyan.paint("INFO"),
        log::Level::Debug => Colour::Blue.paint("DEBUG"),
        log::Level::Trace => Colour::Fixed(245).paint("TRACE"),
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        // Не нужно фильтровать после использования ‘set_max_level’.
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        // Начало
        let open = Colour::Fixed(243).paint("[");
        // Уровень
        let level = level(record.level());
        // Конечная скобка
        let close = Colour::Fixed(243).paint("]");

        eprintln!("{}{} {}{} {}", open, level, record.target(), close, record.args());
    }

    fn flush(&self) {
        // no need to flush with ‘eprintln!’.
    }
}

/// Настраиваем логирование, изменяя уровень логирования на основании переменной окружения
pub fn configure<T: AsRef<OsStr>>(ev: Option<T>) {
    // Есть ли вообще переменная?
    let ev = match ev {
        Some(v)  => v,
        None     => return,
    };

    // Может быть она пустая?
    let env_var = ev.as_ref();
    if env_var.is_empty() {
        return;
    }

    if env_var == "trace" {
        log::set_max_level(log::LevelFilter::Trace);
    }
    else {
        log::set_max_level(log::LevelFilter::Debug);
    }

    // Выставляем глобальный логгер
    let result = log::set_logger(GLOBAL_LOGGER);
    if let Err(e) = result {
        eprintln!("Failed to initialise logger: {}", e);
    }
}