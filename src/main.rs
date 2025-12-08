#![no_std]
#![no_main]

mod vga_buffer;
mod keyboard;
mod shell;
mod parser;
mod history;
mod simple_string;
use core::panic::PanicInfo;
use bootloader::{entry_point, BootInfo};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// env!("CARGO_PKG_NAME")
// env!("CARGO_PKG_DESCRIPTION")
// env!("CARGO_PKG_AUTHORS")
// env!("CARGO_PKG_REPOSITORY")

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    shell::bootstrap(VERSION);
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
