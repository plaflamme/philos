#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)] // https://os.phil-opp.com/cpu-exceptions/
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod gdt;
pub mod interrupts;
pub mod qemu;
#[macro_use]
pub mod serial;
pub mod vga_buffer;

#[cfg(test)]
#[no_mangle]
/// Entrypoint for cargo test
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

pub fn init() {
    interrupts::init_idt();
    interrupts::init_pics();
    gdt::init_gdt();
    x86_64::instructions::interrupts::enable();
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
    loop {}
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

#[cfg(test)]
mod test {
    #[test_case]
    fn test_intr_bkpt() {
        // invoke the breakpoint
        x86_64::instructions::interrupts::int3();
    }
}
