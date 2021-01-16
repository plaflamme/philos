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

                self.buffer.0[row][col] = VgaChar {
                    ascii_char: value,
                    color_code: self.color_code,
                };
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

    fn new_line(&mut self) {
        unimplemented!()
    }
}

// const VGA_BUFFER: &mut Buffer = unsafe { &mut *(0xb8000 as *mut Buffer) };

pub fn test() {

    let mut writer = Writer {
        current_col: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };
    writer.write_u8(b'H');
    writer.write_str("ello ");
    writer.write_str("Wörld!");
}
