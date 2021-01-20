#![no_std]
#![no_main]
// https://os.phil-opp.com/testing/#custom-test-frameworks
#![feature(custom_test_frameworks)]
#![test_runner(philos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use philos::println;
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    philos::init();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut _mapper = unsafe { philos::memory::init(phys_offset) };
    let mut _allocator = unsafe { philos::memory::BootInfoFrameAllocator::new(&boot_info) };

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
