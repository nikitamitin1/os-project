use core::{arch::asm, cell::UnsafeCell};

#[repr(u8)]
pub enum Color {
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

pub fn scancode_to_ascii(sc: u8) -> Option<u8> {
    match sc {
        0x02 => Some(b'1'),
        0x03 => Some(b'2'),
        0x04 => Some(b'3'),
        0x05 => Some(b'4'),
        0x06 => Some(b'5'),
        0x07 => Some(b'6'),
        0x08 => Some(b'7'),
        0x09 => Some(b'8'),
        0x0A => Some(b'9'),
        0x0B => Some(b'0'),

        0x10 => Some(b'q'),
        0x11 => Some(b'w'),
        0x12 => Some(b'e'),
        0x13 => Some(b'r'),
        0x14 => Some(b't'),
        0x15 => Some(b'y'),
        0x16 => Some(b'u'),
        0x17 => Some(b'i'),
        0x18 => Some(b'o'),
        0x19 => Some(b'p'),

        0x1E => Some(b'a'),
        0x1F => Some(b's'),
        0x20 => Some(b'd'),
        0x21 => Some(b'f'),
        0x22 => Some(b'g'),
        0x23 => Some(b'h'),
        0x24 => Some(b'j'),
        0x25 => Some(b'k'),
        0x26 => Some(b'l'),

        0x2C => Some(b'z'),
        0x2D => Some(b'x'),
        0x2E => Some(b'c'),
        0x2F => Some(b'v'),
        0x30 => Some(b'b'),
        0x31 => Some(b'n'),
        0x32 => Some(b'm'),

        0x39 => Some(b' '),   // пробел
        0x1C => Some(b'\n'),  // Enter

        0x0F => Some(b'\t'),  // Tab

        0x0E => Some(0x08), // Backspace

        _ => None,
    }
}

#[derive(Clone, Copy)]
pub struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> Self {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

pub fn get_color_code(foreground: Color, background: Color) -> ColorCode {
    ColorCode::new(foreground, background)
}

#[derive(Clone, Copy)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!(
    "in al, dx",
    in("dx") port,
    out("al") value,
    options(nomem, nostack, preserves_flags),
    );
    value
}

unsafe fn outb(port: u16, value: u8) {
    asm!(
    "out dx, al",
    in("dx") port,
    in("al") value,
    options(nomem, nostack, preserves_flags),
    );
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;
pub const VGA_BUFFER_ADDRESS: usize = 0xb8000;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

struct Writer {
    color_code: ColorCode,
    row: usize,
    column: usize,
    buffer: *mut Buffer,
}

impl Writer {
    fn set_cursor_position(&self) {
        let position = self.row * BUFFER_WIDTH + self.column;
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (position & 0xFF) as u8);
            outb(0x3D4, 0x0E);
            outb(0x3D5, ((position >> 8) & 0xFF) as u8);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row;
                let col = self.column;
                let buffer_ptr = self.buffer;
                unsafe {
                    (*buffer_ptr).chars[row][col] = ScreenChar {
                        ascii_character: byte,
                        color_code: self.color_code,
                    };
                }
                self.column += 1;
            }
        }

        self.set_cursor_position();
    }

    fn new_line(&mut self) {
        if self.row < BUFFER_HEIGHT - 1 {
            self.row += 1;
            self.column = 0;
            return;
        }

        let buffer_ptr = self.buffer;
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    let character = (*buffer_ptr).chars[row][col];
                    (*buffer_ptr).chars[row - 1][col] = character;
                }
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.row = BUFFER_HEIGHT - 1;
        self.column = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        let buffer_ptr = self.buffer;
        for col in 0..BUFFER_WIDTH {
            unsafe {
                (*buffer_ptr).chars[row][col] = blank;
            }
        }
    }
}

struct GlobalWriter(UnsafeCell<Writer>);

impl GlobalWriter {
    const fn new(writer: Writer) -> Self {
        Self(UnsafeCell::new(writer))
    }

    fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Writer) -> R,
    {
        // SAFETY: OS kernel runs on a single core without preemption yet.
        unsafe { f(&mut *self.0.get()) }
    }
}

unsafe impl Sync for GlobalWriter {}

static WRITER: GlobalWriter = GlobalWriter::new(Writer {
    color_code: ColorCode::new(Color::White, Color::Black),
    row: 0,
    column: 0,
    buffer: VGA_BUFFER_ADDRESS as *mut Buffer,
});

pub fn write_byte(byte: u8, color_code: ColorCode) {
    WRITER.with(|writer| {
        writer.color_code = color_code;
        writer.write_byte(byte);
    });
}
