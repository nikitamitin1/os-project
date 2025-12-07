use core::arch::asm;

use crate::vga_buffer;
use crate::vga_buffer::{get_color_code, Color};

const KBD_DATA_PORT: u16 = 0x60;
const KBD_STATUS_PORT: u16 = 0x64;

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

unsafe fn keyboard_has_data() -> bool {
    let status = inb(KBD_STATUS_PORT);
    (status & 0x01) != 0 // if one we have byte for reading
}

unsafe fn read_scancode() -> u8 {
    while !keyboard_has_data(){
    }
    inb(KBD_DATA_PORT)
}

pub fn read_scancode_safe() -> u8 {
    unsafe {read_scancode()}
}

