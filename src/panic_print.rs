//! Panic printer scaffolding.
//!
//! Goal: print panic information (message, file:line, backtrace if any)
//! to VGA and serial. Keep it simple and robust.
//!
//! Topics to read:
//! - `core::panic::PanicInfo` API: message(), location()
//! - Formatting without allocation, avoiding re-entrancy
//! - Possibly disabling interrupts while printing

use core::panic::PanicInfo;
use crate::println;

/// Print panic info. You can call this from `#[panic_handler]`.
pub fn print(info: &PanicInfo) {
    // Minimal safe printing; expand as needed.
    if let Some(loc) = info.location() {
        println!(
            "KERNEL PANIC at {}:{}:{}",
            loc.file(),
            loc.line(),
            loc.column()
        );
        crate::serial::write_str("KERNEL PANIC\n");
    } else {
        println!("KERNEL PANIC at <unknown location>");
        crate::serial::write_str("KERNEL PANIC at <unknown location>\n");
    }
    // Note: formatting via println! avoids heap.
    println!("message: {}", info.message());
    crate::serial::write_str("panic: see VGA for details\n");
    // TODO: Optionally mirror to serial::write_str and add more context.
}
