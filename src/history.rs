use crate::simple_string::FixedString;

const MAX_ENTRIES: usize = 32;
const ENTRY_CAPACITY: usize = 128;

pub struct InputHistory {
    entries: [FixedString<ENTRY_CAPACITY>; MAX_ENTRIES],
    len: usize,
    head: usize,
    cursor: usize,
}

impl InputHistory {
    pub fn new() -> Self {
        Self {
            entries: core::array::from_fn(|_| FixedString::new()),
            len: 0,
            head: 0,
            cursor: 0,
        }
    }

    pub fn push(&mut self, line: &str) {
        if line.is_empty() {
            self.reset_navigation();
            return;
        }

        if self.len > 0 {
            if let Some(last) = self.latest() {
                if last == line {
                    self.reset_navigation();
                    return;
                }
            }
        }

        let target = if self.len < MAX_ENTRIES {
            let idx = (self.head + self.len) % MAX_ENTRIES;
            self.len += 1;
            idx
        } else {
            let idx = self.head;
            self.head = (self.head + 1) % MAX_ENTRIES;
            idx
        };

        self.entries[target].clear();
        let _ = self.entries[target].push_str(line);
        self.cursor = self.len;
    }

    pub fn previous(&mut self) -> Option<&str> {
        if self.len == 0 {
            return None;
        }

        if self.cursor == 0 {
            // already at oldest
        } else if self.cursor > self.len {
            self.cursor = self.len.saturating_sub(1);
        } else {
            self.cursor -= 1;
        }

        self.entry_index(self.cursor)
            .map(|idx| self.entries[idx].as_str())
    }

    pub fn next(&mut self) -> Option<&str> {
        if self.cursor >= self.len {
            self.cursor = self.len;
            return None;
        }

        self.cursor += 1;
        if self.cursor >= self.len {
            self.cursor = self.len;
            return None;
        }

        self.entry_index(self.cursor)
            .map(|idx| self.entries[idx].as_str())
    }

    pub fn latest(&self) -> Option<&str> {
        if self.len == 0 {
            None
        } else {
            let idx = self.entry_index(self.len - 1)?;
            Some(self.entries[idx].as_str())
        }
    }

    pub fn reset_navigation(&mut self) {
        self.cursor = self.len;
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_at_current(&self) -> bool {
        self.cursor >= self.len
    }

    fn entry_index(&self, logical: usize) -> Option<usize> {
        if logical >= self.len || self.len == 0 {
            return None;
        }
        Some((self.head + logical) % MAX_ENTRIES)
    }
}
