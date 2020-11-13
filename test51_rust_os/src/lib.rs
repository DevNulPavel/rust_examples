#![no_std]      // Не используем стандартную библиотеку, а значит никаких стандартных библиотек операционной системы
#![cfg_attr(test, no_main)] // Условный аттрибут, если test - значит main
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use] pub mod vga_buffer;
#[macro_use] pub mod serial;
pub mod qemu;

#[cfg(test)]
use core::{
    panic::{
        PanicInfo
    },
};

// Обработчик паники в тестах, который выводит данные в последовательный порт
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    qemu::exit_qemu(qemu::QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

// Реализуем трейт для всех, кто у нас реализует трейт Fn
impl<T> Testable for T 
where T: Fn() {
    fn run(&self) {
        // Получаем имя нашего типа, вроде как работает на этапе копиляции вообще
        serial_print!("{}...\t", core::any::type_name::<T>());
        // Вызываем непосредственно код
        self();
        // Пишем OK
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    qemu::exit_qemu(qemu::QemuExitCode::Success);
}