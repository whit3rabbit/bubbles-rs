//! Cursor movement and text deletion methods.

use super::model::Model;
use super::types::EchoMode;

impl Model {
    /// Delete all text before the cursor
    pub(super) fn delete_before_cursor(&mut self) {
        if self.pos < self.value.len() {
            self.value.drain(..self.pos);
        } else {
            self.value.clear();
        }
        self.err = self.validate_runes(&self.value);
        self.offset = 0;
        self.set_cursor(0);
    }

    /// Delete all text after the cursor
    pub(super) fn delete_after_cursor(&mut self) {
        self.value.truncate(self.pos);
        self.err = self.validate_runes(&self.value);
        self.set_cursor(self.value.len());
    }

    /// Delete the word before the cursor
    pub(super) fn delete_word_backward(&mut self) {
        if self.pos == 0 || self.value.is_empty() {
            return;
        }

        if self.echo_mode != EchoMode::EchoNormal {
            self.delete_before_cursor();
            return;
        }

        let old_pos = self.pos;

        // Move to start of current word
        if self.pos > 0 {
            self.pos -= 1;

            // Skip whitespace
            while self.pos > 0 && self.value[self.pos].is_whitespace() {
                self.pos -= 1;
            }

            // Skip non-whitespace
            while self.pos > 0 && !self.value[self.pos].is_whitespace() {
                self.pos -= 1;
            }

            if self.pos > 0 {
                self.pos += 1; // Keep one space
            }
        }

        self.value.drain(self.pos..old_pos);
        self.err = self.validate_runes(&self.value);
    }

    /// Delete the word after the cursor
    pub(super) fn delete_word_forward(&mut self) {
        if self.pos >= self.value.len() || self.value.is_empty() {
            return;
        }

        if self.echo_mode != EchoMode::EchoNormal {
            self.delete_after_cursor();
            return;
        }

        let old_pos = self.pos;
        let mut end_pos = self.pos;

        // Skip whitespace
        while end_pos < self.value.len() && self.value[end_pos].is_whitespace() {
            end_pos += 1;
        }

        // Skip non-whitespace
        while end_pos < self.value.len() && !self.value[end_pos].is_whitespace() {
            end_pos += 1;
        }

        self.value.drain(old_pos..end_pos);
        self.err = self.validate_runes(&self.value);
        self.set_cursor(old_pos);
    }

    /// Move cursor backward by one word
    pub(super) fn word_backward(&mut self) {
        if self.pos == 0 || self.value.is_empty() {
            return;
        }

        if self.echo_mode != EchoMode::EchoNormal {
            self.cursor_start();
            return;
        }

        let mut i = self.pos;
        if i > 0 {
            i -= 1;

            // Skip whitespace
            while i > 0 && self.value[i].is_whitespace() {
                self.set_cursor(self.pos - 1);
                i -= 1;
            }

            // Skip non-whitespace
            while i > 0 && !self.value[i].is_whitespace() {
                self.set_cursor(self.pos - 1);
                i -= 1;
            }
        }
    }

    /// Move cursor forward by one word
    pub(super) fn word_forward(&mut self) {
        if self.pos >= self.value.len() || self.value.is_empty() {
            return;
        }

        if self.echo_mode != EchoMode::EchoNormal {
            self.cursor_end();
            return;
        }

        let mut i = self.pos;

        // Skip whitespace
        while i < self.value.len() && self.value[i].is_whitespace() {
            self.set_cursor(self.pos + 1);
            i += 1;
        }

        // Skip non-whitespace
        while i < self.value.len() && !self.value[i].is_whitespace() {
            self.set_cursor(self.pos + 1);
            i += 1;
        }
    }
}
