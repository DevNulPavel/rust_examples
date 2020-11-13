#![no_std]      // Не используем стандартную библиотеку, а значит никаких стандартных библиотек операционной системы
#![no_main]     // Отключаем стандартную точку входа main Rust, которая вызывыется из библиотеки crt после инициализации запуска
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use] mod vga_buffer;

use core::{
    panic::{
        PanicInfo
    }
};

// Данная функция будет вызываться в случае паники
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
    }
}

// Данная функция является точкой входа нашей операционки, поэтому имя _start
// Не занимаемся манглингом функции, экспортируем как есть имя
// Данная функция не должна возвращать никакой результат и никогда не должны выходить из нее
//      поэтому возвращается !
#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    {
        test_main();
    }

    #[cfg(not(test))]
    {
        /*// Наша выводимая строка
        static HELLO: &[u8] = b"Hello World!";

        // Буффер для вывода на экран расположен по фиксированному адресу
        let vga_buffer = 0xb8000 as *mut u8;

        // Ссылку можно тоже развернуть в сам объект с помощью &
        for (i, &byte) in HELLO.iter().enumerate() {
            unsafe {
                // Записываем непосредственно байт куда надо со смещением в 2 байта
                *vga_buffer.offset(i as isize * 2) = byte;
                // Записываем цвет следующим байтом
                *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
            }
        }*/

        vga_buffer::test_print_something();
        //println!("TEST");
        //panic!("Test panic");
    }

    loop {
    }
}

// Для запуска тестов надо убрать в Cargo.toml
// [profile.dev]
// panic = "abort"
#[cfg(test)]
mod test{
    #[test_case]
    fn trivial_assertion() {
        print!("trivial assertion... ");
        assert_eq!(1, 1);
        println!("[ok]");
    }
}


#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    for test in tests {
        test();
    }
}