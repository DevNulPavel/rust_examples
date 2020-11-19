#![no_std]      // Не используем стандартную библиотеку, а значит никаких стандартных библиотек операционной системы
#![no_main]     // Отключаем стандартную точку входа main Rust, которая вызывыется из библиотеки crt после инициализации запуска
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[macro_use] mod vga_buffer;
#[macro_use] mod serial;
mod qemu;
mod panic;
mod interrupts;
mod gdt;
#[cfg(test)] mod test;

use x86_64::{
    registers::{
        control::{
            Cr3
        }
    }
};

////////////////////////////////////////////////////////////////////////

// Специальный переход процессора в режим сна до тех пор, пока не прилетит прерывание
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// Данная функция является точкой входа нашей операционки, поэтому имя _start
// Не занимаемся манглингом функции, экспортируем как есть имя
// Данная функция не должна возвращать никакой результат и никогда не должны выходить из нее
//      поэтому возвращается !
#[cfg(not(test))] // new attribute
#[no_mangle]
pub extern "C" fn _start() -> ! {
    gdt::init();
    interrupts::init();

    // invoke a breakpoint exception
    //x86_64::instructions::interrupts::int3(); // new

    // Вызывает исключение типа page fault
    /*unsafe {
        *(0xdeadbeef as *mut u64) = 42;
    };*/

    // Вызываем переполнение стека
    /*#[allow(unconditional_recursion)]
    fn stack_overflow() {
        stack_overflow(); // for each recursion, the return address is pushed
        volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
    }
    stack_overflow();*/

    // Вызываем краш по памяти, прерывание PageFault
    /*let ptr = 0xdeadbeaf as *mut u32;
    unsafe { let x = *ptr; } // Читать можно без проблем при этом
    unsafe { *ptr = 42; }*/

    // Читаем таблицу 4го уровня виртуальной памяти
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at: {:?}", level_4_page_table.start_address());
    
    println!("TEST TEXT");
    
    hlt_loop();
}


#[cfg(test)] // new attribute
#[no_mangle]
pub extern "C" fn _start() -> ! {
    gdt::init();
    interrupts::init();
    
    test_main();
    
    hlt_loop();
}
