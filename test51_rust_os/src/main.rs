#![no_std]      // Не используем стандартную библиотеку, а значит никаких стандартных библиотек операционной системы
#![no_main]     // Отключаем стандартную точку входа main Rust, которая вызывыется из библиотеки crt после инициализации запуска
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{
    panic::{
        PanicInfo
    }
};
#[allow(unused_imports)]
use rust_os::{
    qemu,
    println,
    serial_println,
    serial_print
};

// Данная функция будет вызываться в случае паники
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
    }
}

// Обработчик паники в тестах, который выводит данные в последовательный порт
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    qemu::exit_qemu(qemu::QemuExitCode::Failed);
    loop {}
}

// Данная функция является точкой входа нашей операционки, поэтому имя _start
// Не занимаемся манглингом функции, экспортируем как есть имя
// Данная функция не должна возвращать никакой результат и никогда не должны выходить из нее
//      поэтому возвращается !
#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    println!("TEST TEXT");

    loop {}
}