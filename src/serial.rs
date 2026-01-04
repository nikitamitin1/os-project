//! Serial (COM1) driver for early diagnostics/logging.
//!
//! Что делает модуль:
//! - Инициализирует 16550‑совместимый UART на COM1 (0x3F8) под 115200 8N1.
//! - Даёт блокирующую запись байта и строки (поллинг по LSR.THR_EMPTY).
//!
//! Что почитать, чтобы понимать код:
//! - 16550 UART: регистры LCR/IER/FCR/MCR/LSR и бит DLAB.
//! - Делитель скорости: базовая тактовая 1_843_200/16 = 115_200 бод; divisor=115_200/baud.
//! - Порты COM1: база 0x3F8; смещения DLL/DLM/LCR/LSR и др.

#![allow(dead_code)]

// COM1 base I/O port
const COM1: u16 = 0x3F8;

// Register offsets from COM1 base
const RBR_THR_DLL: u16 = 0; // Receive Buffer / Transmit Holding / Divisor Latch Low
const IER_DLM: u16 = 1;     // Interrupt Enable / Divisor Latch High
const FCR_IIR: u16 = 2;     // FIFO Control / Interrupt Identification (read)
const LCR: u16 = 3;         // Line Control
const MCR: u16 = 4;         // Modem Control
const LSR: u16 = 5;         // Line Status

// LCR bits
const LCR_WORDLEN_8: u8 = 0b11; // 8 data bits
const LCR_STOP_1: u8 = 0 << 2; // 1 stop bit
const LCR_PARITY_NONE: u8 = 0 << 3;
const LCR_DLAB: u8 = 1 << 7; // Divisor Latch Access Bit

// LSR bits
const LSR_THR_EMPTY: u8 = 1 << 5; // Transmitter Holding Register Empty

/// Инициализация COM1 на 115200 8N1, включение FIFO, MCR: DTR|RTS|OUT2.
///
/// Порядок инициализации (важно):
/// 1) Отключить UART‑прерывания (IER=0)
/// 2) Установить DLAB=1, выставить делитель DLL/DLM
/// 3) Убрать DLAB, включить 8N1 (LCR=0x03)
/// 4) Включить FIFO и очистить очереди (FCR=0xC7)
/// 5) MCR=0x0B (DTR|RTS|OUT2) – OUT2 нужно, если когда‑нибудь включим IRQ
pub fn init_unsafe_16550_default() {
    unsafe {
        use crate::interrupts::{inb, outb};
        let _ = inb; // suppress unused warnings on some platforms

        // 1) Disable UART interrupts
        outb(COM1 + IER_DLM, 0x00);

        // 2) Enable DLAB and program divisor for 115200 baud -> divisor = 1
        outb(COM1 + LCR, LCR_DLAB);
        outb(COM1 + RBR_THR_DLL, 0x01); // DLL
        outb(COM1 + IER_DLM, 0x00);     // DLM

        // 3) 8 data bits, 1 stop, no parity (DLAB=0)
        outb(COM1 + LCR, LCR_WORDLEN_8 | LCR_STOP_1 | LCR_PARITY_NONE); // 0x03

        // 4) Enable FIFO, clear RX/TX, trigger level 14 bytes
        outb(COM1 + FCR_IIR, 0xC7);

        // 5) Modem Control: DTR | RTS | OUT2
        outb(COM1 + MCR, 0x0B);
    }
}

/// Блокирующая передача байта: ждём LSR.THR_EMPTY и пишем в THR.
pub fn write_byte_blocking(byte: u8) {
    unsafe {
        use crate::interrupts::{inb, outb};
        while inb(COM1 + LSR) & LSR_THR_EMPTY == 0 {}
        outb(COM1 + RBR_THR_DLL, byte);
    }
}

/// Запись строки. Для совместимости терминалов переводим '\n' в "\r\n".
pub fn write_str(s: &str) {
    for b in s.bytes() {
        if b == b'\n' {
            write_byte_blocking(b'\r');
        }
        write_byte_blocking(b);
    }
}

/// Необязательный вспомогательный метод: готов ли передатчик.
pub fn is_transmit_empty() -> bool {
    unsafe { (crate::interrupts::inb(COM1 + LSR) & LSR_THR_EMPTY) != 0 }
}
