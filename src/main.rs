#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod vga_buffer;
mod keyboard;
mod shell;
mod parser;
mod history;
mod simple_string;
mod interrupts;
mod paging;
mod gdt;         // GDT + TSS + IST for robust exception handling
// Scaffolding modules for upcoming features. You can hook them up gradually.
mod serial;      // COM1 (16550) serial output – TODO implement init/write
mod logger;      // log::Log backend – TODO route logs to VGA/serial
mod exceptions;  // Exception handlers (page fault, GPF, etc.) – TODO register in IDT
mod time;        // PIT timer + uptime ticks – TODO program PIT and count ticks
mod panic_print; // Panic pretty-printer – TODO print panic info to screen/serial

use core::panic::PanicInfo;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use bootloader::{entry_point, BootInfo};
use paging::FrameAllocator;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const KERNEL_MAP_LIMIT: u64 = 0x0020_0000;

// env!("CARGO_PKG_NAME")
// env!("CARGO_PKG_DESCRIPTION")
// env!("CARGO_PKG_AUTHORS")
// env!("CARGO_PKG_REPOSITORY")

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Load GDT/TSS early so IDT can reference valid selectors/IST.
    gdt::init();
    interrupts::init();
    init_paging(boot_info);
    // Initialize serial/logging
    serial::init_unsafe_16550_default();
    // logger::init(log::LevelFilter::Info);
    // Program PIT for periodic timer ticks.
    time::init_pit(100);
    shell::bootstrap(VERSION, boot_info);
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // TODO: Print panic information in a readable way (screen + serial).
    // panic_print::print(_info);
    loop {}
}

fn init_paging(boot_info: &'static BootInfo) {
    let phys_offset = boot_info.physical_memory_offset;
    let mut frame_allocator = boot_info_frame_allocator(boot_info);

    // Use existing bootloader page tables to avoid breaking current mappings.
    let current_cr3 = unsafe { paging::read_cr3_phys() };
    let mut mapper = unsafe { paging::Mapper::from_existing(current_cr3, phys_offset) };
    unsafe {
        let flags = paging::flags::PRESENT | paging::flags::WRITABLE;
        // Низкая память (2 МиБ) для ранних обращений/BIOS площадок
        mapper.identity_map_range(0, KERNEL_MAP_LIMIT, flags, &mut frame_allocator);
        // VGA текстовый буфер 0xB8000..0xBA000
        mapper.identity_map_range(0x000B_8000, 0x000B_A000, flags, &mut frame_allocator);
        // CR3 не трогаем: дополнили существующие таблицы загрузчика
    }
}

fn boot_info_frame_allocator(boot_info: &'static BootInfo) -> BootInfoFrameAllocator {
    BootInfoFrameAllocator::new(&boot_info.memory_map)
}

fn max_physical_end(boot_info: &'static BootInfo) -> u64 {
    boot_info
        .memory_map
        .iter()
        .map(|region| region.range.end_addr())
        .max()
        .unwrap_or(0)
}

struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    region_index: usize,
    next_addr: u64,
}

impl BootInfoFrameAllocator {
    fn new(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            region_index: 0,
            next_addr: 0,
        }
    }

    fn next_frame_address(&mut self) -> Option<u64> {
        while self.region_index < self.memory_map.len() {
            let region = &self.memory_map[self.region_index];
            if region.region_type != MemoryRegionType::Usable || region.range.is_empty() {
                self.region_index += 1;
                self.next_addr = 0;
                continue;
            }

            let region_start = align_up(region.range.start_addr());
            let region_end = region.range.end_addr();

            if self.next_addr < region_start {
                self.next_addr = region_start;
            }

            if self.next_addr < region_end {
                let frame = self.next_addr;
                self.next_addr += paging::FRAME_SIZE;
                return Some(frame);
            } else {
                self.region_index += 1;
                self.next_addr = 0;
            }
        }
        None
    }
}

impl FrameAllocator for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<u64> {
        self.next_frame_address()
    }
}

const fn align_up(addr: u64) -> u64 {
    let mask = paging::FRAME_SIZE - 1;
    (addr + mask) & !mask
}
