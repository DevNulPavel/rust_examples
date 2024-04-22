use x86_64::{
    structures::{
        tss::{
            TaskStateSegment
        },
        gdt::{
            GlobalDescriptorTable, 
            Descriptor,
            SegmentSelector
        }
    },
    instructions::{
        segmentation::{
            set_cs
        },
        tables::{
            load_tss
        }
    },
    VirtAddr
};
use lazy_static::{
    lazy_static
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // Размер стека обработчика прерывания
            const STACK_SIZE: usize = 4096 * 5;
            // Непосредственно сам стек, переменная статическая, чтобы был доступ всегда и везде
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // Начало стека и конец
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;

            stack_end
        };
        tss
    };

    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    GDT.0.load();
    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}