// Если архитектура такая, тогда подключаем модуль в виде файла конкретного пути
#[cfg(target_arch = "aarch64")]
#[path = "_arch/aarch64/cpu.rs"]
mod arch_cpu;

// Экспортируем функции
pub use arch_cpu::*;
