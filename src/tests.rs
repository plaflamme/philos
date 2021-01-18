use crate::qemu;
use core::panic::PanicInfo;

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

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    qemu::exit(qemu::ExitCode::Failure);
    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Test]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    qemu::exit(qemu::ExitCode::Success);
}

