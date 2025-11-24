#![no_std]
#![no_main]

use core::{arch::asm, panic::PanicInfo};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        clear_screen();
        write_str(b"Type your name and press Enter.\n> ", PROMPT_COLOR);
    }

    let mut input = [0u8; 32];
    let len = read_line(&mut input);

    unsafe {
        write_str(b"\nHello, ", RESPONSE_COLOR);
        write_str(&input[..len], RESPONSE_COLOR);
        write_str(b"!\n", RESPONSE_COLOR);
    }

    loop {}
}

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;
const BUFFER_SIZE: usize = BUFFER_WIDTH * BUFFER_HEIGHT;
const PROMPT_COLOR: u8 = 0x0f;
const INPUT_COLOR: u8 = 0x0a;
const RESPONSE_COLOR: u8 = 0x0b;

static mut CURSOR_POS: usize = 0;

unsafe fn clear_screen() {
    for i in 0..BUFFER_SIZE {
        let offset = (i * 2) as isize;
        VGA_BUFFER.offset(offset).write_volatile(b' ');
        VGA_BUFFER
            .offset(offset + 1)
            .write_volatile(PROMPT_COLOR);
    }
    CURSOR_POS = 0;
}

unsafe fn write_byte(byte: u8, color: u8) {
    if byte == b'\n' {
        newline();
        return;
    }

    if CURSOR_POS >= BUFFER_SIZE {
        return;
    }

    let offset = (CURSOR_POS * 2) as isize;
    VGA_BUFFER.offset(offset).write_volatile(byte);
    VGA_BUFFER.offset(offset + 1).write_volatile(color);
    CURSOR_POS += 1;
}

unsafe fn write_str(data: &[u8], color: u8) {
    for &byte in data {
        write_byte(byte, color);
    }
}

unsafe fn backspace(color: u8) {
    if CURSOR_POS == 0 {
        return;
    }
    CURSOR_POS -= 1;
    let offset = (CURSOR_POS * 2) as isize;
    VGA_BUFFER.offset(offset).write_volatile(b' ');
    VGA_BUFFER.offset(offset + 1).write_volatile(color);
}

unsafe fn newline() {
    CURSOR_POS = ((CURSOR_POS / BUFFER_WIDTH) + 1) * BUFFER_WIDTH;
    if CURSOR_POS >= BUFFER_SIZE {
        CURSOR_POS = (BUFFER_HEIGHT - 1) * BUFFER_WIDTH;
    }
}

fn read_line(buffer: &mut [u8]) -> usize {
    let mut len = 0;

    loop {
        let scancode = read_scancode();

        if scancode & 0x80 != 0 {
            continue;
        }

        match scancode {
            0x1C => {
                unsafe {
                    write_byte(b'\n', PROMPT_COLOR);
                }
                break;
            }
            0x0E => {
                if len > 0 {
                    len -= 1;
                    unsafe {
                        backspace(INPUT_COLOR);
                    }
                }
            }
            _ => {
                if let Some(ch) = scancode_to_ascii(scancode) {
                    if len < buffer.len() {
                        buffer[len] = ch;
                        len += 1;
                        unsafe {
                            write_byte(ch, INPUT_COLOR);
                        }
                    }
                }
            }
        }
    }

    len
}

fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    let byte = match scancode {
        0x02 => b'1',
        0x03 => b'2',
        0x04 => b'3',
        0x05 => b'4',
        0x06 => b'5',
        0x07 => b'6',
        0x08 => b'7',
        0x09 => b'8',
        0x0A => b'9',
        0x0B => b'0',
        0x10 => b'q',
        0x11 => b'w',
        0x12 => b'e',
        0x13 => b'r',
        0x14 => b't',
        0x15 => b'y',
        0x16 => b'u',
        0x17 => b'i',
        0x18 => b'o',
        0x19 => b'p',
        0x1E => b'a',
        0x1F => b's',
        0x20 => b'd',
        0x21 => b'f',
        0x22 => b'g',
        0x23 => b'h',
        0x24 => b'j',
        0x25 => b'k',
        0x26 => b'l',
        0x2C => b'z',
        0x2D => b'x',
        0x2E => b'c',
        0x2F => b'v',
        0x30 => b'b',
        0x31 => b'n',
        0x32 => b'm',
        0x39 => b' ',
        _ => return None,
    };
    Some(byte)
}

fn read_scancode() -> u8 {
    loop {
        if keyboard_has_data() {
            return unsafe { port_read_u8(0x60) };
        }
    }
}

fn keyboard_has_data() -> bool {
    unsafe { port_read_u8(0x64) & 1 != 0 }
}

unsafe fn port_read_u8(port: u16) -> u8 {
    let value: u8;
    asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    value
}
