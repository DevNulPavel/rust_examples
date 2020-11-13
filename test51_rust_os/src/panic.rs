use core::{
    panic::{
        PanicInfo
    }
};
#[cfg(test)]
use crate::{
    qemu
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