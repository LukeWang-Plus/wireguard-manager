//! Single-line text input widget with cursor-aware editing.

/// A single-line text input widget state with cursor-aware editing.
pub struct TextInput {
    /// Current text content.
    content: String,
    /// Byte-offset cursor position within `content`.
    cursor: usize,
}

impl TextInput {
    /// Create an empty input.
    pub const fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
        }
    }

    /// Create an input pre-filled with `value`, cursor at end.
    pub fn with_value(value: &str) -> Self {
        let len = value.len();
        Self {
            content: value.to_string(),
            cursor: len,
        }
    }

    /// Return the current text content.
    pub fn value(&self) -> &str {
        &self.content
    }

    /// Returns the cursor position as a character count (for display).
    pub fn cursor(&self) -> usize {
        debug_assert!(self.content.is_char_boundary(self.cursor));
        self.content[..self.cursor].chars().count()
    }

    /// Insert a character at the cursor position.
    pub fn insert(&mut self, c: char) {
        debug_assert!(self.content.is_char_boundary(self.cursor));
        self.content.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            debug_assert!(self.content.is_char_boundary(self.cursor));
            let prev = self.content[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(i, _)| i);
            self.content.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    /// Delete the character after the cursor (delete).
    pub fn delete_forward(&mut self) {
        if self.cursor < self.content.len() {
            debug_assert!(self.content.is_char_boundary(self.cursor));
            let next = self.content[self.cursor..]
                .char_indices()
                .nth(1)
                .map_or(self.content.len(), |(i, _)| self.cursor + i);
            self.content.drain(self.cursor..next);
        }
    }

    /// Move the cursor one character to the left.
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            debug_assert!(self.content.is_char_boundary(self.cursor));
            self.cursor = self.content[..self.cursor]
                .char_indices()
                .next_back()
                .map_or(0, |(i, _)| i);
        }
    }

    /// Move the cursor one character to the right.
    pub fn move_right(&mut self) {
        if self.cursor < self.content.len() {
            debug_assert!(self.content.is_char_boundary(self.cursor));
            self.cursor = self.content[self.cursor..]
                .char_indices()
                .nth(1)
                .map_or(self.content.len(), |(i, _)| self.cursor + i);
        }
    }

    /// Move the cursor to the beginning of the input.
    pub const fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// Move the cursor to the end of the input.
    pub const fn move_end(&mut self) {
        self.cursor = self.content.len();
    }

    /// Clear all text and reset the cursor to position 0.
    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
    }
}

#[cfg(test)]
mod tests;
