use crate::cpu;
use core::panic::PanicInfo;

// В случае паники мы уходим в бесконечный цикл
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cpu::wait_forever()
}
