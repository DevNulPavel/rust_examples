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
use crate::{
    //print,
    println
};

lazy_static! {
    // Мы можем инициализировать статическую переменную в несколько шагов
    // просто оборачивая все в блок кода
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
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