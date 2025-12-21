//! Extremely small paging helper: identity mapping and frame allocation.
//!
//! This is intentionally minimal — just enough to build our own page tables
//! without relying on advanced abstractions yet.

#![allow(dead_code)]

use core::arch::asm;

/// Number of entries per x86_64 page table.
pub const ENTRIES_PER_TABLE: usize = 512;

/// Common flag bits for 4 KiB pages.
pub mod flags {
    pub const PRESENT: u64 = 1 << 0;
    pub const WRITABLE: u64 = 1 << 1;
    pub const USER: u64 = 1 << 2;
    pub const HUGE_PAGE: u64 = 1 << 7;
    pub const NO_EXECUTE: u64 = 1 << 63;
}

const ADDRESS_MASK: u64 = 0x000F_FFFF_FFFF_F000;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn set(&mut self, frame: u64, flags: u64) {
        debug_assert_eq!(frame & 0xFFF, 0);
        self.0 = (frame & ADDRESS_MASK) | (flags & !ADDRESS_MASK);
    }

    pub fn is_unused(&self) -> bool {
        (self.0 & flags::PRESENT) == 0
    }

    pub fn addr(&self) -> u64 {
        self.0 & ADDRESS_MASK
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; ENTRIES_PER_TABLE],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); ENTRIES_PER_TABLE],
        }
    }

    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.clear();
        }
    }

    pub fn entry_mut(&mut self, index: usize) -> &mut PageTableEntry {
        assert!(index < ENTRIES_PER_TABLE);
        &mut self.entries[index]
    }
}

pub const FRAME_SIZE: u64 = 4096;

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<u64>;
}

pub struct BumpFrameAllocator {
    next: u64,
    end: u64,
}

impl BumpFrameAllocator {
    pub fn new(start: u64, end: u64) -> Self {
        Self {
            next: align_up(start),
            end,
        }
    }
}

impl FrameAllocator for BumpFrameAllocator {
    fn allocate_frame(&mut self) -> Option<u64> {
        if self.next >= self.end {
            return None;
        }
        let frame = self.next;
        self.next = align_up(self.next + FRAME_SIZE);
        Some(frame)
    }
}

const fn align_up(addr: u64) -> u64 {
    (addr + FRAME_SIZE - 1) & !(FRAME_SIZE - 1)
}

pub struct Mapper {
    pml4_phys: u64,
    phys_offset: u64,
}

impl Mapper {
    /// Create a mapper for the current page tables without clearing them.
    pub unsafe fn from_existing(pml4_phys: u64, phys_offset: u64) -> Self {
        Self { pml4_phys, phys_offset }
    }
    pub unsafe fn new(pml4_phys: u64, phys_offset: u64) -> Self {
        unsafe {
            let virt = (pml4_phys + phys_offset) as *mut PageTable;
            (*virt).zero();
        }
        Self {
            pml4_phys,
            phys_offset,
        }
    }

    pub unsafe fn identity_map_range(
        &mut self,
        start: u64,
        end: u64,
        flags: u64,
        allocator: &mut impl FrameAllocator,
    ) {
        let mut addr = start & !0xFFF;
        let end = align_up(end);
        while addr < end {
            unsafe {
                self.map_page(addr, addr, flags, allocator);
            }
            addr += FRAME_SIZE;
        }
    }

    pub unsafe fn map_page(
        &mut self,
        virt: u64,
        phys: u64,
        flags: u64,
        allocator: &mut impl FrameAllocator,
    ) {
        let mut table = self.pml4_phys;
        for &index in &[
            pml4_index(virt),
            pdpt_index(virt),
            pd_index(virt),
        ] {
            table = unsafe { self.ensure_next_table(table, index, allocator) };
        }
        let last = unsafe { self.table_mut(table) };
        let entry = last.entry_mut(pt_index(virt));
        if entry.is_unused() {
            entry.set(phys, flags);
        } // else: already mapped, keep existing mapping
    }

    pub unsafe fn activate(&self) {
        let pml4 = self.pml4_phys;
        unsafe {
            asm!(
                "mov cr3, {0}",
                in(reg) pml4,
                options(nostack, preserves_flags),
            );
            let mut cr0: u64;
            asm!("mov {0}, cr0", out(reg) cr0, options(nostack, preserves_flags));
            if cr0 & (1 << 31) == 0 {
                cr0 |= 1 << 31;
                asm!("mov cr0, {0}", in(reg) cr0, options(nostack));
            }
        }
    }

    unsafe fn table_mut(&self, phys: u64) -> &mut PageTable {
        let virt = (phys + self.phys_offset) as *mut PageTable;
        &mut *virt
    }

    unsafe fn ensure_next_table(
        &self,
        table_phys: u64,
        index: usize,
        allocator: &mut impl FrameAllocator,
    ) -> u64 {
        let table = self.table_mut(table_phys);
        let entry = table.entry_mut(index);
        if entry.is_unused() {
            let frame = allocator
                .allocate_frame()
                .expect("no free frames for paging structures");
            let next = self.table_mut(frame);
            next.zero();
            entry.set(frame, flags::PRESENT | flags::WRITABLE);
            frame
        } else {
            entry.addr()
        }
    }
}

#[inline]
pub unsafe fn read_cr3_phys() -> u64 {
    let value: u64;
    core::arch::asm!("mov {0}, cr3", out(reg) value, options(nostack, preserves_flags));
    // Mask out PCID/PWT/PCD low bits; keep only physical base (bits 63:12 on x86_64)
    value & 0x000F_FFFF_FFFF_F000
}

fn pml4_index(addr: u64) -> usize {
    ((addr >> 39) & 0x1FF) as usize
}

fn pdpt_index(addr: u64) -> usize {
    ((addr >> 30) & 0x1FF) as usize
}

fn pd_index(addr: u64) -> usize {
    ((addr >> 21) & 0x1FF) as usize
}

fn pt_index(addr: u64) -> usize {
    ((addr >> 12) & 0x1FF) as usize
}
