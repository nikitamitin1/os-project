//! Minimal fixed-capacity string builder for the OS project.
//!
//! Inspired by `heapless::String`, but tiny and purpose-built for
//! assembling short messages without pulling in `alloc::String`.

/// Errors that can occur when pushing into a [`FixedString`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedStringError {
    NoCapacity,
}

/// `FixedString<N>` stores at most `N` bytes of UTF-8 text without heap allocations.
pub struct FixedString<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> FixedString<N> {
    /// Create an empty `FixedString`.
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    /// Return the current length in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return total capacity.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Clear the buffer without altering capacity.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Append a single byte (must be valid UTF-8 if you plan to call [`as_str`]).
    pub fn push_byte(&mut self, byte: u8) -> Result<(), FixedStringError> {
        if self.len >= N {
            return Err(FixedStringError::NoCapacity);
        }
        self.buf[self.len] = byte;
        self.len += 1;
        Ok(())
    }

    /// Append a UTF-8 string slice.
    pub fn push_str(&mut self, s: &str) -> Result<(), FixedStringError> {
        for byte in s.bytes() {
            self.push_byte(byte)?;
        }
        Ok(())
    }

    /// Access the underlying string (only valid if inputs were UTF-8).
    pub fn as_str(&self) -> &str {
        // Safety: we only ever push bytes coming from valid UTF-8 `&str`
        // or ASCII bytes that also yield valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }
}
