#![no_std]      // Не используем стандартную библиотеку, а значит никаких стандартных библиотек операционной системы
#![no_main]     // Отключаем стандартную точку входа main Rust, которая вызывыется из библиотеки crt после инициализации запуска
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use] mod vga_buffer;
#[macro_use] mod serial;
mod qemu;
mod panic;
mod test;


////////////////////////////////////////////////////////////////////////

// Данная функция является точкой входа нашей операционки, поэтому имя _start
// Не занимаемся манглингом функции, экспортируем как есть имя
// Данная функция не должна возвращать никакой результат и никогда не должны выходить из нее
//      поэтому возвращается !
#[cfg(not(test))] // new attribute
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("TEST TEXT");
    loop {}
}


#[cfg(test)] // new attribute
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}
