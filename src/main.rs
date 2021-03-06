#![no_std]
#![no_main]
// https://os.phil-opp.com/testing/#custom-test-frameworks
#![feature(custom_test_frameworks)]
#![test_runner(philos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use philos::println;
use philos::task::executor::Executor;
use philos::task::keyboard::print_keypresses;
use philos::task::Task;

extern crate alloc;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    philos::init();
    unsafe { philos::memory::init(boot_info) };
    philos::allocator::init().expect("heap allocation failed");
    let acpi = unsafe { philos::acpi::init() }.expect("unable to enable ACPI");

    println!("ACPI revision {}", acpi.revision);
    if let Ok(platform_info) = acpi.platform_info() {
        println!("Power profile  : {:?}", platform_info.power_profile);
        println!("Interrupt model: {:?}", platform_info.interrupt_model);
        if let Some(processor_info) = platform_info.processor_info {
            println!("Boot processor : {:?}", processor_info.boot_processor);
            for proc in processor_info.application_processors.iter() {
                println!("Appl processor : {:?}", proc);
            }
        }
    }

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(print_keypresses()));
    executor.run();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    philos::serial_println!("{}", info);
    philos::hlt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    philos::test_panic_handler(info);
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
