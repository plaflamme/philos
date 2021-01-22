#![no_std]
#![cfg_attr(test, no_main)]
#![feature(alloc_error_handler)] // https://os.phil-opp.com/heap-allocation/#the-alloc-error-handler-attribute
#![feature(const_in_array_repeat_expressions)] // https://os.phil-opp.com/allocator-designs/#implementation-2
#![feature(const_mut_refs)] // https://os.phil-opp.com/allocator-designs/#implementation-1
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)] // https://os.phil-opp.com/cpu-exceptions/
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod qemu;
#[macro_use]
pub mod serial;
pub mod task;
pub mod vga_buffer;

extern crate alloc;

#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

/// Entrypoint for cargo test
#[cfg(test)]
fn test_kernel_main(_: &'static bootloader::BootInfo) -> ! {
    init();
    test_main();
    hlt();
}

pub fn init() {
    interrupts::init_idt();
    interrupts::init_pics();
    gdt::init_gdt();
    x86_64::instructions::interrupts::enable();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub trait Test {
    fn run(&self) -> ();
}

impl<T> Test for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    qemu::exit(qemu::ExitCode::Failure);
    hlt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn test_runner(tests: &[&dyn Test]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    qemu::exit(qemu::ExitCode::Success);
}

pub fn hlt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
mod test {
    #[test_case]
    fn test_intr_bkpt() {
        // invoke the breakpoint
        x86_64::instructions::interrupts::int3();
    }
}
