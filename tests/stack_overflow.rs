#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)] // https://os.phil-opp.com/cpu-exceptions/

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use philos::{qemu, serial_print, serial_println};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(philos::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

extern "x86-interrupt" fn double_fault_handler(_: &mut InterruptStackFrame, _: u64) -> ! {
    serial_println!("[ok]");
    qemu::exit(qemu::ExitCode::Success);
    loop{}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    philos::gdt::init_gdt();
    TEST_IDT.load();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    philos::test_panic_handler(info);
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(&0).read(); // prevent tail recursion optimizations
}
