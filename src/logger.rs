//! Logger scaffolding: implement `log::Log` to route logs to VGA/serial.
//!
//! Plan:
//! - Implement `log` to format records minimally and write to VGA and/or COM1.
//! - Keep it allocation-free; use small fixed buffers if needed.
//! - Initialize early in `kernel_main` with `logger::init(LevelFilter::Info)`.
//!
//! Topics to read:
//! - `log` crate in `no_std` mode (static logger)
//! - Formatting without allocation (core::fmt::Write/VGA writer)
//! - Serial output (see `serial` module)

#![allow(dead_code)]

use core::fmt::Write as _;

struct KernelLogger;

static LOGGER: KernelLogger = KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        // TODO: refine filtering policy if necessary.
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // TODO: Route to VGA and/or serial. For now, keep it minimal.
        // Example approach:
        // 1) Build a small line: "LEVEL: message\n"
        // 2) Write to VGA via crate::print!/println!
        // 3) Optionally mirror to serial::write_str
        let _level = record.level();
        let _msg = record.args();

        // Placeholder no-op to keep compilation clean:
        let _ = (_level, _msg);
    }

    fn flush(&self) {}
}

/// Initialize global logger and set max level.
pub fn init(level: log::LevelFilter) {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(level);
}

/// Optional helper: log a line directly without macros.
pub fn log_line(level: log::Level, msg: &str) {
    if log::log_enabled!(level) {
        // TODO: send to VGA/serial; for now, discard.
        let _ = msg;
    }
}

