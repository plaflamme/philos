use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(sf: &mut InterruptStackFrame) {
    println!("Exception - Breakpoint");
    println!("{:#?}", sf);
}

extern "x86-interrupt" fn double_fault_handler(sf: &mut InterruptStackFrame, error_code: u64) -> ! {
    panic!("Exception - DoubleFault ({})\n{:#?}", error_code, sf)
}
