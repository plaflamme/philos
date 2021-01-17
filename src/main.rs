#![no_std]
#![no_main]

// https://os.phil-opp.com/testing/#custom-test-frameworks
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

mod vga_buffer;
mod qemu;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("{} {}", "Hello", "world!");

    #[cfg(test)]
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    qemu::exit(qemu::ExitCode::Success);
}
