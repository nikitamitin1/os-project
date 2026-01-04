//! Exception handlers scaffolding (x86_64).
//!
//! Handlers you likely want early:
//! - Page Fault (#PF, vector 14): read CR2, decode error code, print info.
//! - General Protection Fault (#GP, vector 13): print error code/state.
//! - Double Fault (#DF, vector 8): requires IST for robust handling.
//!
//! Topics to read:
//! - IDT gate types (interrupt vs trap gate) and DPL
//! - Error code bits for #PF/#GP; CR2 for faulting linear address
//! - TSS + IST stack for #DF
//! - iretq frame layout in long mode
//!
//! Wiring instructions:
//! - In `interrupts::init()`, set IDT entries for vectors 8,13,14 to these handlers.
//! - Consider using trap gate (0x8F00) for some exceptions.

use core::{hint::spin_loop, sync::atomic::{AtomicBool, Ordering}};
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::registers::control::Cr2;

use crate::println;

// Arm this flag to trigger a nested #PF inside the #PF handler, which
// will escalate to a #DF handled on a dedicated IST stack.
static TRIGGER_DF_ON_PF: AtomicBool = AtomicBool::new(false);

/// Public helper to arm DF test and cause a first page fault.
/// A second fault will be issued from within the PF handler.
pub fn trigger_double_fault_via_page_fault() {
    TRIGGER_DF_ON_PF.store(true, Ordering::SeqCst);
    unsafe {
        // Use a high canonical address that's very unlikely to be mapped.
        let ptr = 0x4000_0000_0000 as *const u64;
        core::ptr::read_volatile(ptr);
    }
}

/// Double Fault handler — usually `-> !` since recovery is rare without IST.
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    // Mirror to VGA and serial, then halt the CPU in a low-power loop.
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    crate::serial::write_str("EXCEPTION: DOUBLE FAULT\n");
    loop { x86_64::instructions::hlt() }
}

/// General Protection Fault handler (vector 13).
pub extern "x86-interrupt" fn gpf_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // Mirror minimal info to serial as well.
    println!("EXCEPTION: GENERAL PROTECTION FAULT, ec={:#x}\n{:#?}", error_code, stack_frame);
    crate::serial::write_str("EXCEPTION: GENERAL PROTECTION FAULT\n");
    loop { spin_loop() }
}

/// Page Fault handler (vector 14).
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    // Faulting linear address is in CR2
    let addr = Cr2::read_raw();
    let p = (error_code & 1) != 0;          // 0=not-present, 1=protection
    let wr = (error_code & (1 << 1)) != 0;  // 0=read, 1=write
    let us = (error_code & (1 << 2)) != 0;  // 0=supervisor, 1=user
    let rsv = (error_code & (1 << 3)) != 0; // reserved-bit violation
    let id = (error_code & (1 << 4)) != 0;  // instruction fetch

    println!(
        "EXCEPTION: PAGE FAULT @ {:#x}, ec={:#x} P={} WR={} US={} RSVD={} ID={}",
        addr, error_code, p, wr, us, rsv, id
    );
    crate::serial::write_str("EXCEPTION: PAGE FAULT\n");
    println!("{:#?}", stack_frame);
    // If armed, deliberately cause another PF while already handling PF to raise #DF.
    if TRIGGER_DF_ON_PF.swap(false, Ordering::SeqCst) {
        unsafe {
            let ptr = 0x4000_0000_0000 as *const u64;
            core::ptr::read_volatile(ptr);
        }
    }
    // Fatal by default: do not resume faulting instruction
    loop { spin_loop() }
}
