//! Interrupt subsystem setup.
//!
//! Sets up a minimal IDT with a breakpoint and keyboard handler,
//! remaps the legacy PIC, and exposes low-level helpers to access
//! the PIC/data ports.

use core::{arch::asm, ptr};
use x86_64;
use x86_64::instructions::segmentation::Segment;
use x86_64::registers::segmentation::CS;
use x86_64::structures::idt::InterruptStackFrame;

use crate::{exceptions, keyboard, println};


/// Represents the layout of a single IDT entry (interrupt gate).
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct IdtEntry {
    pub offset_low: u16,
    pub selector: u16,
    pub options: u16,
    pub offset_mid: u16,
    pub offset_high: u32,
    pub reserved: u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            options: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    pub fn new(handler: usize) -> Self {
        let mut entry = Self::missing();
        entry.set_handler(handler);
        entry.selector = CS::get_reg().0;
        entry.options = 0x8E00; // present, ring0, interrupt gate
        entry
    }

    /// Create an interrupt gate entry and assign an IST (1..=7). 0 disables IST.
    pub fn new_with_ist(handler: usize, ist: u8) -> Self {
        let mut entry = Self::new(handler);
        let ist = (ist as u16) & 0x0007;
        entry.options = (entry.options & !0x0007) | ist;
        entry
    }

    fn set_handler(&mut self, handler: usize) {
        self.offset_low = handler as u16;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
    }
}

/// IDT descriptor passed to `lidt`.
#[repr(C, packed)]
pub struct Idtr {
    pub limit: u16,
    pub base: u64,
}

const IDT_LEN: usize = 256;
static mut IDT: [IdtEntry; IDT_LEN] = [IdtEntry::missing(); IDT_LEN];

/// Initialize the interrupt subsystem.
pub fn init() {
    unsafe {
        IDT[InterruptIndex::Breakpoint as usize] =
            IdtEntry::new(breakpoint_handler as *const () as usize);
        IDT[InterruptIndex::Timer as usize] =
            IdtEntry::new(timer_interrupt_handler as *const () as usize);
        IDT[InterruptIndex::Keyboard as usize] =
            IdtEntry::new(keyboard_interrupt_handler as *const () as usize);

        // Register key exception handlers
        IDT[14] = IdtEntry::new(exceptions::page_fault_handler as *const () as usize);
        IDT[13] = IdtEntry::new(exceptions::gpf_handler as *const () as usize);
        // После реализации GDT+TSS включаем #DF с IST=1
        IDT[8] = IdtEntry::new_with_ist(
            exceptions::double_fault_handler as *const () as usize,
            crate::gdt::DOUBLE_FAULT_IST_INDEX_FOR_IDT as u8,
        );
        // IDT[13] = IdtEntry::new(exceptions::gpf_handler as usize);

        remap_pic();
        load_idt(ptr::addr_of!(IDT).cast(), IDT_LEN);
    }

    x86_64::instructions::interrupts::enable();
}

/// Load the IDT using the `lidt` instruction.
unsafe fn load_idt(idt: *const IdtEntry, len: usize) {
    let idtr = Idtr {
        limit: ((len * core::mem::size_of::<IdtEntry>()) - 1) as u16,
        base: idt as u64,
    };
    unsafe {
        asm!(
            "lidt [{}]",
            in(reg) &idtr,
            options(nostack, preserves_flags),
        );
    }
}

/// Send End-Of-Interrupt to the Programmable Interrupt Controller.
pub unsafe fn send_eoi(irq: u8) {
    if irq >= 0x28 {
        unsafe { outb(0xA0, 0x20) }; // Slave PIC
    }
    unsafe { outb(0x20, 0x20) }; // Master PIC
}

/// Remap the legacy PIC.
unsafe fn remap_pic() {
    unsafe {
        outb(0x20, 0x11);
        io_wait();
        outb(0xA0, 0x11);
        io_wait();

        outb(0x21, 0x20);
        io_wait();
        outb(0xA1, 0x28); 
        io_wait();

        outb(0x21, 4);
        io_wait();
        outb(0xA1, 2);
        io_wait();

        outb(0x21, 0x01);
        io_wait();
        outb(0xA1, 0x01);
        io_wait();

        outb(0x21, 0x00);
        outb(0xA1, 0x00);
    }
}

/// A small wait function for PIC communication.
fn io_wait() {
    unsafe {
        outb(0x80, 0);
    }
}

pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags),
        );
    }
}

pub unsafe fn inb(port: u16) -> u8 {
    let mut value: u8;
    unsafe {
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nomem, nostack, preserves_flags),
        );
    }
    value
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Breakpoint = 0x03,
    Timer = 0x20,
    Keyboard = 0x21,
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame,
) {
    let scancode: u8 = unsafe { inb(0x60) };
    keyboard::push_scancode(scancode);
    unsafe {
        send_eoi(InterruptIndex::Keyboard as u8);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        send_eoi(InterruptIndex::Timer as u8);
    }
    crate::time::tick();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
