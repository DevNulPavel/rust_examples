// Подключаем ассемблерный код
global_asm!(include_str!("cpu.S"));

// Функция бесконечного цикла
#[inline(always)]
pub fn wait_forever() -> ! {
    unsafe {
        // Бесконечный цикл
        loop {
            #[rustfmt::skip]
            // Запускаем функцию wfe
            asm!(
                "wfe",
                options(nomem, nostack, preserves_flags)
            );
        }
    }
}
