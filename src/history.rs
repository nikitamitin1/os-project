//! Input history buffer skeleton.
//!
//! This module is intentionally left unfinished so you can practice
//! implementing a simple shell history (like pressing the up/down
//! arrows in a terminal).

use crate::simple_string::FixedString;

/// Maximum number of history entries to store.
const MAX_ENTRIES: usize = 64;
const N: usize = 128;

pub struct InputHistory {
    entries: [Option<FixedString<N>>; MAX_ENTRIES],
    len: usize,
    cursor: usize,
}

impl InputHistory {
    /// Create an empty history buffer.
    ///
    /// * Initialise your storage fields.
    /// * Reset cursors so the first `prev()` call knows nothing is stored yet.
    pub fn new() -> Self {
        Self {
            entries: [(); MAX_ENTRIES].map(|_| None),
            len: 0,
            cursor: 0,
        }
    }

    /// Push a new line into history.
    ///
    /// * If the buffer is full, drop the oldest entry (circular buffer).
    /// * Reset the navigation cursor so UP starts from the newest line.
    pub fn push(&mut self, _line: &str) {
        self.entries[self.cursor] = Some(FixedString::new());
        if let Some(entry) = &mut self.entries[self.cursor] {
            let _ = entry.push_str(_line);
        }
        
        if self.len < MAX_ENTRIES {
            self.len += 1;
        } else {
            // Buffer is full, move cursor to overwrite the oldest entry
            self.cursor = (self.cursor + 1) % MAX_ENTRIES;
        }


    }

    /// Navigate backwards (like pressing the UP arrow).
    ///
    /// * Move the cursor towards older entries.
    /// * Return `Some(&str)` (or `Option<String>` if you prefer owned data) with the line.
    /// * If no older entry exists, return `None`.
    pub fn previous(&mut self) -> Option<&str> {
        self.cursor = self.cursor.wrapping_sub(1);
        if let Some(entry) = &self.entries[self.cursor] {
            Some(entry.as_str())
        } else {
            None
        }
    }

    /// Navigate forwards (like pressing the DOWN arrow).
    ///
    /// # TODO
    /// * Move the cursor towards newer entries.
    /// * Once you reach the most recent line, return `None` so the shell can show the
    ///   "current" unfinished input.
    pub fn next(&mut self) -> Option<&str> {
        todo!("return the next stored command, if any")
    }

    /// Reset the navigation cursor back to the "current" line (no history selection).
    ///
    /// # TODO
    /// * Call this whenever the user edits the line manually so the next UP arrow
    ///   starts from the newest entry again.
    pub fn reset_navigation(&mut self) {
        todo!("set the cursor so history navigation restarts from the newest entry")
    }
}
