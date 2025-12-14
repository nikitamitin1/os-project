use core::cell::UnsafeCell;
use x86_64::instructions::interrupts;

const SCANCODE_QUEUE_CAPACITY: usize = 256;

struct ScancodeQueue {
    buffer: [u8; SCANCODE_QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    len: usize,
}

impl ScancodeQueue {
    const fn new() -> Self {
        Self {
            buffer: [0; SCANCODE_QUEUE_CAPACITY],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    fn push(&mut self, scancode: u8) {
        if self.len == SCANCODE_QUEUE_CAPACITY {
            return;
        }
        self.buffer[self.head] = scancode;
        self.head = (self.head + 1) % SCANCODE_QUEUE_CAPACITY;
        self.len += 1;
    }

    fn pop(&mut self) -> Option<u8> {
        if self.len == 0 {
            return None;
        }

        let byte = self.buffer[self.tail];
        self.tail = (self.tail + 1) % SCANCODE_QUEUE_CAPACITY;
        self.len -= 1;
        Some(byte)
    }
}

struct SharedQueue(UnsafeCell<ScancodeQueue>);

impl SharedQueue {
    const fn new() -> Self {
        Self(UnsafeCell::new(ScancodeQueue::new()))
    }

    fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ScancodeQueue) -> R,
    {
        interrupts::without_interrupts(|| unsafe { f(&mut *self.0.get()) })
    }
}

unsafe impl Sync for SharedQueue {}

static QUEUE: SharedQueue = SharedQueue::new();

/// Called from the keyboard interrupt handler to enqueue the latest scancode.
pub fn push_scancode(scancode: u8) {
    QUEUE.with(|queue| queue.push(scancode));
}

/// Pops the next pending scancode if available.
pub fn pop_scancode() -> Option<u8> {
    QUEUE.with(|queue| queue.pop())
}
