use x86_64::{
    structures::{
        idt::{
            InterruptDescriptorTable,
            InterruptStackFrame
        }
    }
};
use lazy_static::{
    lazy_static
};
use pic8259_simple::{
    ChainedPics
};
use spin::{
    Mutex
};
use crate::{
    gdt::{
        DOUBLE_FAULT_IST_INDEX
    },
    println
};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { 
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) 
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    // Мы можем инициализировать статическую переменную в несколько шагов
    // просто оборачивая все в блок кода
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Прерывание брейкпоинта
        idt
            .breakpoint
            .set_handler_fn(breakpoint_handler);
            
        unsafe {
            // Прерывание краша
            idt
                .double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX); // Выставляем отдельный стек для обработчика прерывания

            // Прерывание таймера
            idt[InterruptIndex::Timer.as_usize()]
                .set_handler_fn(timer_interrupt_handler); // new
        }
        idt
    };
}

pub fn init() {
    IDT.load();
    unsafe { 
        PICS.lock()
            .initialize();
    }
    // Включаем обработку аппаратных прерываний нашим прроцессором
    x86_64::instructions::interrupts::enable();
}

// Соглашение о вызове специальное для исключений, параметры передаются через стек? 
extern "x86-interrupt" 
fn breakpoint_handler(stack_frame: &mut InterruptStackFrame){
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" 
fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" 
fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame){
    print!(".");

    unsafe {
        // Говорим контроллеру прерываний, что мы успешно обработали прерывание,
        // можем дальше продолжать обрабатывать прерывания
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}