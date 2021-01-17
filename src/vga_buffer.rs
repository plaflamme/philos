use core::fmt;
use core::ptr;
use spin::Mutex;

use lazy_static::lazy_static;
use core::fmt::Write;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        current_col: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)] // u4 if it existed
enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(transparent)] // so the representation is that of the field inside
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> Self {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
struct VgaChar {
    ascii_char: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer([[VgaChar; BUFFER_WIDTH]; BUFFER_HEIGHT]);

pub struct Writer {
    current_col: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_u8(&mut self, value: u8) {
        match value {
            b'\n' => self.new_line(),
            _ => {
                if self.current_col >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.current_col;

                let vga_color = VgaChar {
                    ascii_char: value,
                    color_code: self.color_code,
                };
                self.write_at(row, col, vga_color);
                self.current_col += 1;
            },
        }
    }

    pub fn write_str(&mut self, str: &str) {
        for byte in str.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_u8(byte),
                _ => self.write_u8(0xfe), // non-printable
            };
        }
    }

    fn write_at(&mut self, row: usize, col: usize, v: VgaChar) {
        unsafe {
            // write_volatile guarantees that this call will not be optimized away.
            // The volatile crate could be used but we only have one instance at this point.
            ptr::write_volatile(&mut self.buffer.0[row][col], v);
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let v = self.buffer.0[row][col];
                self.write_at(row - 1, col, v);
            }
        }
        let clear = VgaChar {
            ascii_char: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.write_at(BUFFER_HEIGHT - 1, col, clear);
        }
        self.current_col = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => ($crate::vga_buffer::_print(format_args!($($args)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($args:tt)*) => ($crate::print!("{}\n", format_args!($($args)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}
