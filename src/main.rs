#![no_std]
#![no_main]

// https://os.phil-opp.com/testing/#custom-test-frameworks
#![feature(custom_test_frameworks)]
#![test_runner(philos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use philos::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("{} {}", "Hello", "world!");

    philos::init();

    #[cfg(test)]
    test_main();

    philos::hlt();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    philos::hlt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    philos::test_panic_handler(info);
}
