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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidDigit,
    EmptyInput,
    InvalidSign,
    BufferTooSmall,
}

impl ParseError {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParseError::InvalidDigit => "input contains a non-digit character",
            ParseError::EmptyInput => "input string is empty",
            ParseError::InvalidSign => "invalid sign placement",
            ParseError::BufferTooSmall => "buffer too small for conversion",
        }
    }
}

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
pub fn int_to_str_buf(value: i64, buf: &mut [u8]) -> Result<&str, ParseError> {
    if value == 0 {
        if buf.is_empty() { return Err(ParseError::BufferTooSmall); }
        buf[0] = b'0';
        return Ok(core::str::from_utf8(&buf[..1]).unwrap());
    }

    let negative = value < 0;
    let mut n = if negative { value.wrapping_neg() as u64 } else { value as u64 };

    let mut i = 0;
    while n > 0 {
        if i >= buf.len() { return Err(ParseError::BufferTooSmall); }
        let digit = (n % 10) as u8;
        buf[i] = b'0' + digit;
        n /= 10;
        i += 1;
    }

    if negative {
        if i >= buf.len() { return Err(ParseError::BufferTooSmall); }
        buf[i] = b'-';
        i += 1;
    }

    buf[..i].reverse();
    Ok(core::str::from_utf8(&buf[..i]).unwrap())
}
