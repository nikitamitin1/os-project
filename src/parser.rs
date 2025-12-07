//! Simple parsing helpers for the toy OS shell.
//!
//! This module is intentionally left mostly unimplemented so you can
//! practice writing conversion logic between textual user input and
//! numeric data types.

/// Parse an integer from an ASCII string slice.
///
/// # TODO
/// * Decide whether to support optional `+`/`-` signs.
/// * Handle decimal digits only (hex/bin support can come later).
/// * Validate that every character is a digit before converting.
/// * Return either the parsed integer or an error describing why parsing failed.

use crate::vga_buffer;

pub fn parse_int_from_str(_s: &str) -> Result<i64, ParseError> {
    let bytes = _s.as_bytes();

    if bytes.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let (sign, digits) = match bytes[0] {
        b'+' => (1, &bytes[1..]),
        b'-' => (-1, &bytes[1..]),
        _ => (1, bytes),
    };

    if digits.is_empty() {
        return Err(ParseError::InvalidDigit);
    }

    let mut value: i64 = 0;
    for &byte in digits {
        if byte < b'0' || byte > b'9' {
            return Err(ParseError::InvalidDigit);
        }
        value = value.checked_mul(10).and_then(|v| v.checked_add((byte - b'0') as i64)).ok_or(ParseError::InvalidDigit)?;
    }

    Ok(value * sign)
}

/// Convert an integer back to its decimal ASCII representation.
///
/// # TODO
/// * Support negative numbers (leading `-`).
/// * Avoid using `format!` – build the string manually for practice.
/// * Decide whether to trim/keep leading zeros.
/// * Consider reusing buffers instead of allocating each call (optional).
pub fn int_to_string(_value: i64) -> Result<usize, ParseError> {
    if _value == 0 {
        Ok(b'0' as usize)
    }

    else {
        let mut value = _value;
        let mut buffer = [0u8; 20]; // Enough for i64
        let mut index = 0;

        if value < 0 {
            value = -value;
            // Handle negative sign if needed
        }

        while value > 0 {
            let digit = (value % 10) as u8;
            buffer[index] = b'0' + digit;
            index += 1;
            value /= 10;
        }

        // Reverse the buffer to get the correct order
        for i in 0..index / 2 {
            buffer.swap(i, index - 1 - i);
        }

        Ok(buffer.len())
}

/// Minimal error type for the parsing helpers above.
///
/// # TODO
/// * Extend with more variants once you know the edge cases you want
///   to catch (overflow, empty string, invalid digit, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidDigit,
    EmptyInput,
    InvalidSign
}

