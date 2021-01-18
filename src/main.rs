#![no_std]
#![no_main]

// https://os.phil-opp.com/testing/#custom-test-frameworks
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod qemu;
#[macro_use]
mod serial;
#[cfg(test)]
mod tests;
mod vga_buffer;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("{} {}", "Hello", "world!");

    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(not(test))]
use core::panic::PanicInfo;
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

