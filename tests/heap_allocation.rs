#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(philos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::VirtAddr;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    philos::init();

    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { philos::memory::init(phys_offset) };
    let mut allocator = unsafe { philos::memory::BootInfoFrameAllocator::new(&boot_info) };

    philos::allocator::init(&mut mapper, &mut allocator).expect("failure");
    test_main();
    philos::hlt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    philos::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..philos::allocator::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
