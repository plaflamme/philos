#![no_std]
#![no_main]

// https://os.phil-opp.com/testing/#custom-test-frameworks
#![feature(custom_test_frameworks)]
#![test_runner(philos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use philos::println;
use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;
use x86_64::structures::paging::Translate;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    philos::init();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { philos::memory::init(phys_offset) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

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
