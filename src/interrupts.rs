use crate::{print, println, hlt};
use lazy_static::lazy_static;
use pic8259_simple::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_interrupt_handler);

        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(sf: &mut InterruptStackFrame) {
    println!("Exception - Breakpoint");
    println!("{:#?}", sf);
}

extern "x86-interrupt" fn double_fault_handler(sf: &mut InterruptStackFrame, error_code: u64) -> ! {
    panic!("Exception - DoubleFault ({})\n{:#?}", error_code, sf)
}

extern "x86-interrupt" fn timer_interrupt_handler(_: &mut InterruptStackFrame) {
    print!(".");
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8) };
}

extern "x86-interrupt" fn page_fault_interrupt_handler(sf: &mut InterruptStackFrame, error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", sf);

    hlt();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_: &mut InterruptStackFrame) {
    // keyboard scancode port
    let mut port = x86_64::instructions::port::Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: spin::Mutex<Keyboard<pc_keyboard::layouts::Us104Key, ScancodeSet1>> = {
            let kb = Keyboard::new(pc_keyboard::layouts::Us104Key, ScancodeSet1, pc_keyboard::HandleControl::Ignore);
            spin::Mutex::new(kb)
        };
    }
    let mut kb = KEYBOARD.lock();
    if let Ok(Some(key_event)) = kb.add_byte(scancode) {
        if let Some(key) = kb.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard as u8) };
}

pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new( unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

pub fn init_pics() {
    unsafe { PICS.lock().initialize() };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET,
    Keyboard,
}
