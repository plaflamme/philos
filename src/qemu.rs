use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
pub enum ExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit(code: ExitCode) {
    let mut isa_exit_port = Port::new(0xf4);
    unsafe {
        isa_exit_port.write(code as u32);
    }
}
