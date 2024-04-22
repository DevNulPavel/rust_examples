use crate::{
    qemu
};

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

#[allow(dead_code)]
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    qemu::exit_qemu(qemu::QemuExitCode::Success);
}