//! Timekeeping scaffolding using the legacy PIT (8253/8254).
//!
//! Plan:
//! - Program PIT channel 0 to a desired frequency (e.g. 100 Hz, mode 3).
//! - On each timer IRQ, call `time::tick()` to increment a global counter.
//! - Provide `uptime_ticks()` and conversions to ms/sec via known HZ.
//!
//! Topics to read:
//! - PIT ports (0x40..0x43), command word, divisor calculation
//! - PIC routing of IRQ0 (we already remap PIC)
//! - Atomic counters in `no_std`

#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

static HZ: AtomicU32 = AtomicU32::new(18); // BIOS default ~18.2 Hz until programmed
static TICKS: AtomicU64 = AtomicU64::new(0);

/// Program PIT channel 0 for a given frequency (in Hz).
///
/// TODO (you implement):
/// - Compute divisor: 1_193_182 / hz (clamp 1..=65535).
/// - Write command to port 0x43 (e.g., 0x36 for ch0, lobyte/hibyte, mode 3).
/// - Write low byte then high byte of divisor to port 0x40.
pub fn init_pit(hz: u32) {
    let _ = hz;
    unsafe {
        use crate::interrupts::outb;
        let clamped = core::cmp::max(1, core::cmp::min(65_535, (1_193_182u32 / hz) as u32));
        outb(0x43, 0x36);
        outb(0x40, (clamped & 0xFF) as u8);
        outb(0x40, (clamped >> 8) as u8);
    }
    HZ.store(hz, Ordering::Relaxed);
}

/// Increment system tick counter; call from timer IRQ handler.
pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Current ticks since boot.
pub fn uptime_ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/// Configured tick rate.
pub fn frequency_hz() -> u32 {
    HZ.load(Ordering::Relaxed)
}

/// Convenience: uptime in milliseconds (integer division).
pub fn uptime_ms() -> u64 {
    let ticks = uptime_ticks();
    let hz = core::cmp::max(1, frequency_hz() as u64);
    (ticks * 1_000) / hz
}

