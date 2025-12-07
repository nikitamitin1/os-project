#![no_std]
#![no_main]
extern crate alloc;

mod vga_buffer;
mod keyboard;
mod shell;
mod parser;

use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// env!("CARGO_PKG_NAME")
// env!("CARGO_PKG_DESCRIPTION")
// env!("CARGO_PKG_AUTHORS")
// env!("CARGO_PKG_REPOSITORY")

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    shell::print_on_entry("Version is VERSION");
    loop {
        }
    }
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
