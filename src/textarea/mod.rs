//! Textarea component for Bubble Tea applications.
//!
//! This module provides a multi-line text input component with feature parity
//! to the Go `bubbles/textarea` package. It supports soft-wrapping, line
//! numbers, customizable prompts, clipboard integration, and rich theming via
//! Lip Gloss styles.
//!
//! The component implements the crate's `Component` trait so it can be focused
//! and blurred, and it exposes a `Model` with idiomatic methods for editing and
//! navigation (insert, delete, move by character/word/line, etc.).
//!
//! ### Features
//! - Soft-wrapped lines with correct column/character accounting for double-width runes
//! - Optional line numbers and per-line prompts (static or via a prompt function)
//! - Cursor movement by character, word, and line with deletion/edit helpers
//! - Viewport-driven rendering for large inputs
//! - Clipboard paste integration (platform dependent)
//! - Theming via `TextareaStyle` for focused and blurred states
//!
//! ### Example
//! ```rust
//! use bubbletea_widgets::{textarea, Component};
//!
//! // Create a textarea with defaults
//! let mut ta = textarea::new();
//! ta.set_width(40);
//! ta.set_height(6);
//! ta.placeholder = "Type here…".into();
//!
//! // Focus to start receiving input
//! let _ = ta.focus();
//!
//! // Programmatic edits
//! ta.insert_string("Hello\nworld!");
//! ta.word_left();
//! ta.uppercase_right();
//!
//! // Render view (string with ANSI styling)
//! let view = ta.view();
//! println!("{}", view);
//! ```
//!
//! See the `helpers` module for key bindings and styling utilities, and
//! `memoization` for the internal soft-wrap cache.

pub mod helpers;
pub mod memoization;

#[cfg(test)]
mod tests;

use helpers::*;
use memoization::MemoizedWrap;

use crate::{cursor, viewport, Component};
use bubbletea_rs::{Cmd, Model as BubbleTeaModel};
use lipgloss_extras::lipgloss;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// Constants matching Go implementation
const MIN_HEIGHT: usize = 1;
const DEFAULT_HEIGHT: usize = 6;
const DEFAULT_WIDTH: usize = 40;
const DEFAULT_CHAR_LIMIT: usize = 0; // no limit
const DEFAULT_MAX_HEIGHT: usize = 99;
const DEFAULT_MAX_WIDTH: usize = 500;
const MAX_LINES: usize = 10000;

/// Internal messages for clipboard operations
#[derive(Debug, Clone)]
pub struct PasteMsg(pub String);

/// Error message produced when a paste operation fails.
#[derive(Debug, Clone)]
pub struct PasteErrMsg(pub String);

/// LineInfo helper for tracking line information regarding soft-wrapped lines
/// Direct port from Go's LineInfo struct
#[derive(Debug, Clone, Default)]
pub struct LineInfo {
    /// Width is the number of columns in the line
    pub width: usize,
    /// CharWidth is the number of characters in the line to account for double-width runes
    pub char_width: usize,
    /// Height is the number of rows in the line  
    pub height: usize,
    /// StartColumn is the index of the first column of the line
    pub start_column: usize,
    /// ColumnOffset is the number of columns that the cursor is offset from the start of the line
    pub column_offset: usize,
    /// RowOffset is the number of rows that the cursor is offset from the start of the line
    pub row_offset: usize,
    /// CharOffset is the number of characters that the cursor is offset from the start of the line
    pub char_offset: usize,
}

/// Model is the Bubble Tea model for this text area element.
/// Direct port from Go's Model struct with all fields preserved
#[derive(Debug)]
pub struct Model {
    // Error state
    /// Optional error string surfaced by the component.
    pub err: Option<String>,

    // General settings - memoization cache
    cache: MemoizedWrap,

    // Display settings
    /// Prompt is printed at the beginning of each line
    pub prompt: String,
    /// Placeholder is the text displayed when the user hasn't entered anything yet
    pub placeholder: String,
    /// ShowLineNumbers, if enabled, causes line numbers to be printed after the prompt
    pub show_line_numbers: bool,
    /// EndOfBufferCharacter is displayed at the end of the input
    pub end_of_buffer_character: char,

    // KeyMap encodes the keybindings recognized by the widget
    /// Key bindings recognized by the widget.
    pub key_map: TextareaKeyMap,

    // Styling. FocusedStyle and BlurredStyle are used to style the textarea in focused and blurred states
    /// Style used when the textarea is focused.
    pub focused_style: TextareaStyle,
    /// Style used when the textarea is blurred.
    pub blurred_style: TextareaStyle,
    // style is the current styling to use - pointer equivalent in Rust
    current_style: TextareaStyle,

    // Cursor is the text area cursor
    /// Embedded cursor model for caret rendering and blinking.
    pub cursor: cursor::Model,

    // Limits
    /// CharLimit is the maximum number of characters this input element will accept
    pub char_limit: usize,
    /// MaxHeight is the maximum height of the text area in rows
    pub max_height: usize,
    /// MaxWidth is the maximum width of the text area in columns
    pub max_width: usize,

    // Dynamic prompt function - Option to handle nullable function pointer
    prompt_func: Option<fn(usize) -> String>,
    /// promptWidth is the width of the prompt
    prompt_width: usize,

    // Dimensions
    /// width is the maximum number of characters that can be displayed at once
    width: usize,
    /// height is the maximum number of lines that can be displayed at once
    height: usize,

    // Content - using Vec<Vec<char>> to match Go's [][]rune
    /// Underlying text value as runes (characters)
    value: Vec<Vec<char>>,

    // State
    /// focus indicates whether user input focus should be on this input component
    focus: bool,
    /// Cursor column
    col: usize,
    /// Cursor row  
    row: usize,
    /// Last character offset, used to maintain state when cursor is moved vertically
    last_char_offset: usize,

    // Viewport is the vertically-scrollable viewport of the multi-line text input
    viewport: viewport::Model,
}

impl Model {
    /// Create a new textarea model with default settings - port of Go's New()
    pub fn new() -> Self {
        let vp = viewport::Model::new(0, 0);
        // Disable viewport key handling to let textarea handle keys (no keymap field in viewport)

        let cur = cursor::Model::new();

        let (focused_style, blurred_style) = default_styles();

        let mut model = Self {
            err: None,
            cache: MemoizedWrap::new(),
            prompt: format!("{} ", lipgloss::thick_border().left),
            placeholder: String::new(),
            show_line_numbers: true,
            end_of_buffer_character: ' ',
            key_map: TextareaKeyMap::default(),
            focused_style: focused_style.clone(),
            blurred_style: blurred_style.clone(),
            current_style: blurred_style, // Start blurred
            cursor: cur,
            char_limit: DEFAULT_CHAR_LIMIT,
            max_height: DEFAULT_MAX_HEIGHT,
            max_width: DEFAULT_MAX_WIDTH,
            prompt_func: None,
            prompt_width: 0,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            value: vec![vec![]; MIN_HEIGHT],
            focus: false,
            col: 0,
            row: 0,
            last_char_offset: 0,
            viewport: vp,
        };

        // Ensure value has minimum height and maxLines capacity
        model.value.reserve(MAX_LINES);
        model.set_height(DEFAULT_HEIGHT);
        model.set_width(DEFAULT_WIDTH);

        model
    }

    /// Set the value of the text input - port of Go's SetValue
    pub fn set_value(&mut self, s: impl Into<String>) {
        self.reset();
        self.insert_string(s.into());
        // After setting full value, position cursor at end of input (last line)
        self.row = self.value.len().saturating_sub(1);
        if let Some(line) = self.value.get(self.row) {
            self.set_cursor(line.len());
        }
    }

    /// Insert a string at the cursor position - port of Go's InsertString
    pub fn insert_string(&mut self, s: impl Into<String>) {
        let s = s.into();
        let runes: Vec<char> = s.chars().collect();
        self.insert_runes_from_user_input(runes);
    }

    /// Insert a rune at the cursor position - port of Go's InsertRune
    pub fn insert_rune(&mut self, r: char) {
        self.insert_runes_from_user_input(vec![r]);
    }

    /// Get the current value as a string - port of Go's Value()
    pub fn value(&self) -> String {
        if self.value.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        for (i, line) in self.value.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.extend(line.iter());
        }
        result
    }

    /// Length returns the number of characters currently in the text input - port of Go's Length()
    pub fn length(&self) -> usize {
        let mut l = 0;
        for row in &self.value {
            l += row
                .iter()
                .map(|&ch| UnicodeWidthChar::width(ch).unwrap_or(0))
                .sum::<usize>();
        }
        // Add newline characters count
        l + self.value.len().saturating_sub(1)
    }

    /// LineCount returns the number of lines currently in the text input - port of Go's LineCount()
    pub fn line_count(&self) -> usize {
        self.value.len()
    }

    /// Line returns the line position - port of Go's Line()
    pub fn line(&self) -> usize {
        self.row
    }

    /// Focused returns the focus state on the model - port of Go's Focused()
    pub fn focused(&self) -> bool {
        self.focus
    }

    /// Reset sets the input to its default state with no input - port of Go's Reset()
    pub fn reset(&mut self) {
        self.value = vec![vec![]; MIN_HEIGHT];
        self.value.reserve(MAX_LINES);
        self.col = 0;
        self.row = 0;
        self.viewport.goto_top();
        self.set_cursor(0);
    }

    /// Width returns the width of the textarea - port of Go's Width()
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height returns the current height of the textarea - port of Go's Height()
    pub fn height(&self) -> usize {
        self.height
    }

    /// SetWidth sets the width of the textarea - port of Go's SetWidth()
    pub fn set_width(&mut self, w: usize) {
        // Update prompt width only if there is no prompt function
        if self.prompt_func.is_none() {
            self.prompt_width = self.prompt.width();
        }

        // Add base style borders and padding to reserved outer width
        let reserved_outer = 0; // Simplified for now, lipgloss API differs

        // Add prompt width to reserved inner width
        let mut reserved_inner = self.prompt_width;

        // Add line number width to reserved inner width
        if self.show_line_numbers {
            let ln_width = 4; // Up to 3 digits for line number plus 1 margin
            reserved_inner += ln_width;
        }

        // Input width must be at least one more than the reserved inner and outer width
        let min_width = reserved_inner + reserved_outer + 1;
        let mut input_width = w.max(min_width);

        // Input width must be no more than maximum width
        if self.max_width > 0 {
            input_width = input_width.min(self.max_width);
        }

        self.viewport.width = input_width.saturating_sub(reserved_outer);
        self.width = input_width
            .saturating_sub(reserved_outer)
            .saturating_sub(reserved_inner);
    }

    /// SetHeight sets the height of the textarea - port of Go's SetHeight()
    pub fn set_height(&mut self, h: usize) {
        if self.max_height > 0 {
            self.height = clamp(h, MIN_HEIGHT, self.max_height);
            self.viewport.height = clamp(h, MIN_HEIGHT, self.max_height);
        } else {
            self.height = h.max(MIN_HEIGHT);
            self.viewport.height = h.max(MIN_HEIGHT);
        }
    }

    /// SetPromptFunc supersedes the Prompt field and sets a dynamic prompt instead
    /// Port of Go's SetPromptFunc
    pub fn set_prompt_func(&mut self, prompt_width: usize, func: fn(usize) -> String) {
        self.prompt_func = Some(func);
        self.prompt_width = prompt_width;
    }

    /// SetCursor moves the cursor to the given position - port of Go's SetCursor()
    pub fn set_cursor(&mut self, col: usize) {
        self.col = clamp(
            col,
            0,
            self.value.get(self.row).map_or(0, |line| line.len()),
        );
        // Reset last char offset when moving cursor horizontally
        self.last_char_offset = 0;
    }

    /// CursorStart moves the cursor to the start of the input field - port of Go's CursorStart()
    pub fn cursor_start(&mut self) {
        self.set_cursor(0);
    }

    /// CursorEnd moves the cursor to the end of the input field - port of Go's CursorEnd()
    pub fn cursor_end(&mut self) {
        if let Some(line) = self.value.get(self.row) {
            self.set_cursor(line.len());
        }
    }

    /// CursorDown moves the cursor down by one line - port of Go's CursorDown()
    pub fn cursor_down(&mut self) {
        let li = self.line_info();
        let char_offset = self.last_char_offset.max(li.char_offset);
        self.last_char_offset = char_offset;

        if li.row_offset + 1 >= li.height && self.row < self.value.len().saturating_sub(1) {
            self.row += 1;
            self.col = 0;
        } else {
            // Move the cursor to the start of the next line
            const TRAILING_SPACE: usize = 2;
            if let Some(line) = self.value.get(self.row) {
                self.col =
                    (li.start_column + li.width + TRAILING_SPACE).min(line.len().saturating_sub(1));
            }
        }

        let nli = self.line_info();
        self.col = nli.start_column;

        if nli.width == 0 {
            return;
        }

        let mut offset = 0;
        while offset < char_offset {
            if self.row >= self.value.len()
                || self.col >= self.value.get(self.row).map_or(0, |line| line.len())
                || offset >= nli.char_width.saturating_sub(1)
            {
                break;
            }
            if let Some(line) = self.value.get(self.row) {
                if let Some(&ch) = line.get(self.col) {
                    offset += UnicodeWidthChar::width(ch).unwrap_or(0);
                }
            }
            self.col += 1;
        }
    }

    /// CursorUp moves the cursor up by one line - port of Go's CursorUp()
    pub fn cursor_up(&mut self) {
        let li = self.line_info();
        let char_offset = self.last_char_offset.max(li.char_offset);
        self.last_char_offset = char_offset;

        if li.row_offset == 0 && self.row > 0 {
            self.row -= 1;
            if let Some(line) = self.value.get(self.row) {
                self.col = line.len();
            }
        } else {
            // Move the cursor to the end of the previous line
            const TRAILING_SPACE: usize = 2;
            self.col = li.start_column.saturating_sub(TRAILING_SPACE);
        }

        let nli = self.line_info();
        self.col = nli.start_column;

        if nli.width == 0 {
            return;
        }

        let mut offset = 0;
        while offset < char_offset {
            if let Some(line) = self.value.get(self.row) {
                if self.col >= line.len() || offset >= nli.char_width.saturating_sub(1) {
                    break;
                }
                if let Some(&ch) = line.get(self.col) {
                    offset += UnicodeWidthChar::width(ch).unwrap_or(0);
                }
                self.col += 1;
            } else {
                break;
            }
        }
    }

    /// Move to the beginning of input - port of Go's moveToBegin()
    pub fn move_to_begin(&mut self) {
        self.row = 0;
        self.set_cursor(0);
    }

    /// Move to the end of input - port of Go's moveToEnd()
    pub fn move_to_end(&mut self) {
        self.row = self.value.len().saturating_sub(1);
        if let Some(line) = self.value.get(self.row) {
            self.set_cursor(line.len());
        }
    }

    // Internal helper functions matching Go implementation structure

    /// Port of Go's insertRunesFromUserInput
    fn insert_runes_from_user_input(&mut self, mut runes: Vec<char>) {
        // Clean up any special characters in the input
        runes = self.sanitize_runes(runes);

        if self.char_limit > 0 {
            let avail_space = self.char_limit.saturating_sub(self.length());
            if avail_space == 0 {
                return;
            }
            if avail_space < runes.len() {
                runes.truncate(avail_space);
            }
        }

        // Split the input into lines
        let mut lines = Vec::new();
        let mut lstart = 0;

        for (i, &r) in runes.iter().enumerate() {
            if r == '\n' {
                lines.push(runes[lstart..i].to_vec());
                lstart = i + 1;
            }
        }

        if lstart <= runes.len() {
            lines.push(runes[lstart..].to_vec());
        }

        // Obey the maximum line limit
        if MAX_LINES > 0 && self.value.len() + lines.len() - 1 > MAX_LINES {
            let allowed_height = (MAX_LINES - self.value.len() + 1).max(0);
            lines.truncate(allowed_height);
        }

        if lines.is_empty() {
            return;
        }

        // Ensure current row exists
        while self.row >= self.value.len() {
            self.value.push(Vec::new());
        }

        // Save the remainder of the original line at the current cursor position
        let tail = if self.col < self.value[self.row].len() {
            self.value[self.row][self.col..].to_vec()
        } else {
            Vec::new()
        };

        // Paste the first line at the current cursor position
        if self.col <= self.value[self.row].len() {
            self.value[self.row].truncate(self.col);
        }
        self.value[self.row].extend_from_slice(&lines[0]);
        self.col += lines[0].len();

        if lines.len() > 1 {
            // Add the new lines maintaining cursor on the first line's end
            for (i, line) in lines[1..].iter().enumerate() {
                self.value.insert(self.row + 1 + i, line.clone());
            }
            // Move cursor to end of the last inserted line (Go behavior on SetValue)
            self.row += lines.len() - 1;
            self.col = lines.last().map(|l| l.len()).unwrap_or(0);
            // Append tail to current line
            self.value[self.row].extend_from_slice(&tail);
        } else {
            // No newlines: append tail back to current line
            self.value[self.row].extend_from_slice(&tail);
        }

        self.set_cursor(self.col);
    }

    /// Sanitize runes for input - simple version
    fn sanitize_runes(&self, runes: Vec<char>) -> Vec<char> {
        // For now, just return as-is. In Go this handles special characters
        runes
    }

    /// LineInfo returns line information for the current cursor position
    /// Port of Go's LineInfo() - enhanced with better wrapped line handling
    pub fn line_info(&mut self) -> LineInfo {
        if self.row >= self.value.len() {
            return LineInfo::default();
        }

        // Clone the line to avoid borrowing issues
        let current_line = self.value[self.row].clone();
        let width = self.width;
        let grid = self.cache.wrap(&current_line, width);

        // Find out which visual wrap line we are currently on
        let mut counter = 0;
        for (i, line) in grid.iter().enumerate() {
            // Check if cursor is exactly at the end of this wrapped line and should wrap to next
            if counter + line.len() == self.col && i + 1 < grid.len() {
                // We wrap around to the next visual line
                let next_line = &grid[i + 1];
                return LineInfo {
                    char_offset: 0,
                    column_offset: 0,
                    height: grid.len(),
                    row_offset: i + 1,
                    start_column: self.col,
                    width: next_line.len(),
                    char_width: next_line
                        .iter()
                        .map(|&ch| UnicodeWidthChar::width(ch).unwrap_or(0))
                        .sum(),
                };
            }

            // Check if cursor falls within this wrapped line
            if counter + line.len() >= self.col {
                let col_in_line = self.col.saturating_sub(counter);
                let char_off: usize = line
                    .iter()
                    .take(col_in_line.min(line.len()))
                    .map(|&ch| UnicodeWidthChar::width(ch).unwrap_or(0))
                    .sum();

                return LineInfo {
                    char_offset: char_off,
                    column_offset: col_in_line,
                    height: grid.len(),
                    row_offset: i,
                    start_column: counter,
                    width: line.len(),
                    char_width: line
                        .iter()
                        .map(|&ch| UnicodeWidthChar::width(ch).unwrap_or(0))
                        .sum(),
                };
            }

            counter += line.len();
        }

        // Cursor is past the end of all content - place at end of last line
        if let Some(last_line) = grid.last() {
            let last_counter = counter - last_line.len();
            return LineInfo {
                char_offset: last_line
                    .iter()
                    .map(|&ch| UnicodeWidthChar::width(ch).unwrap_or(0))
                    .sum(),
                column_offset: last_line.len(),
                height: grid.len(),
                row_offset: grid.len().saturating_sub(1),
                start_column: last_counter,
                width: last_line.len(),
                char_width: last_line
                    .iter()
                    .map(|&ch| UnicodeWidthChar::width(ch).unwrap_or(0))
                    .sum(),
            };
        }

        LineInfo::default()
    }

    /// Delete before cursor - port of Go's deleteBeforeCursor()
    pub fn delete_before_cursor(&mut self) {
        if let Some(line) = self.value.get_mut(self.row) {
            let tail = if self.col <= line.len() {
                line[self.col..].to_vec()
            } else {
                Vec::new()
            };
            *line = tail;
        }
        self.set_cursor(0);
    }

    /// Delete after cursor - port of Go's deleteAfterCursor()
    pub fn delete_after_cursor(&mut self) {
        if let Some(line) = self.value.get_mut(self.row) {
            line.truncate(self.col);
            let line_len = line.len();
            self.set_cursor(line_len);
        }
    }

    /// Delete character backward - port of Go's deleteCharacterBackward()
    pub fn delete_character_backward(&mut self) {
        self.col = clamp(
            self.col,
            0,
            self.value.get(self.row).map_or(0, |line| line.len()),
        );
        if self.col == 0 {
            self.merge_line_above(self.row);
            return;
        }

        if let Some(line) = self.value.get_mut(self.row) {
            if !line.is_empty() && self.col > 0 {
                line.remove(self.col - 1);
                self.set_cursor(self.col - 1);
            }
        }
    }

    /// Delete character forward - port of Go's deleteCharacterForward()
    pub fn delete_character_forward(&mut self) {
        if let Some(line) = self.value.get_mut(self.row) {
            if !line.is_empty() && self.col < line.len() {
                line.remove(self.col);
            }
        }

        if self.col >= self.value.get(self.row).map_or(0, |line| line.len()) {
            self.merge_line_below(self.row);
        }
    }

    /// Delete word backward - port of Go's deleteWordLeft()
    pub fn delete_word_backward(&mut self) {
        if self.col == 0 {
            self.merge_line_above(self.row);
            return;
        }

        let line = if let Some(line) = self.value.get(self.row) {
            line.clone()
        } else {
            return;
        };

        if line.is_empty() {
            return;
        }

        // Find word boundaries - Go bubbles deleteWordLeft behavior
        let mut start = self.col;
        let mut end = self.col;

        // If we're not at the end of a word, find the end first
        while end < line.len() && line.get(end).is_some_and(|&c| !c.is_whitespace()) {
            end += 1;
        }

        // Find start of the word we're in or before
        while start > 0 && line.get(start - 1).is_some_and(|&c| !c.is_whitespace()) {
            start -= 1;
        }

        // Only include preceding space if cursor is not at end of word
        if self.col < line.len() && line.get(self.col).is_some_and(|&c| !c.is_whitespace()) {
            // Cursor is inside word, include preceding space
            if start > 0 && line.get(start - 1).is_some_and(|&c| c.is_whitespace()) {
                start -= 1;
            }
        }

        if let Some(line_mut) = self.value.get_mut(self.row) {
            let end_clamped = end.min(line_mut.len());
            let start_clamped = start.min(end_clamped);
            line_mut.drain(start_clamped..end_clamped);
        }

        self.set_cursor(start);
    }

    /// Delete word forward - port of Go's deleteWordRight()
    pub fn delete_word_forward(&mut self) {
        let line = if let Some(line) = self.value.get(self.row) {
            line.clone()
        } else {
            return;
        };

        if self.col >= line.len() || line.is_empty() {
            self.merge_line_below(self.row);
            return;
        }

        let old_col = self.col;
        let mut new_col = self.col;

        // Skip whitespace
        while new_col < line.len() {
            if let Some(&ch) = line.get(new_col) {
                if ch.is_whitespace() {
                    new_col += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Skip word characters
        while new_col < line.len() {
            if let Some(&ch) = line.get(new_col) {
                if !ch.is_whitespace() {
                    new_col += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Delete the selected text
        if let Some(line) = self.value.get_mut(self.row) {
            if new_col > line.len() {
                line.truncate(old_col);
            } else {
                line.drain(old_col..new_col);
            }
        }

        self.set_cursor(old_col);
    }

    /// Merge line below - port of Go's mergeLineBelow()
    fn merge_line_below(&mut self, row: usize) {
        if row >= self.value.len().saturating_sub(1) {
            return;
        }

        // Combine the two lines
        if let Some(next_line) = self.value.get(row + 1).cloned() {
            if let Some(current_line) = self.value.get_mut(row) {
                current_line.extend_from_slice(&next_line);
            }
        }

        // Remove the next line by shifting all lines up
        self.value.remove(row + 1);
    }

    /// Merge line above - port of Go's mergeLineAbove()
    fn merge_line_above(&mut self, row: usize) {
        if row == 0 {
            return;
        }

        if let Some(prev_line) = self.value.get(row - 1) {
            self.col = prev_line.len();
        }
        self.row = row - 1;

        // Combine the two lines
        if let Some(current_line) = self.value.get(row).cloned() {
            if let Some(prev_line) = self.value.get_mut(row - 1) {
                prev_line.extend_from_slice(&current_line);
            }
        }

        // Remove the current line
        self.value.remove(row);
    }

    /// Split line - port of Go's splitLine()
    fn split_line(&mut self, row: usize, col: usize) {
        if let Some(line) = self.value.get(row) {
            let head = line[..col].to_vec();
            let tail = line[col..].to_vec();

            // Replace current line with head
            self.value[row] = head;

            // Insert tail as new line
            self.value.insert(row + 1, tail);

            self.col = 0;
            self.row += 1;
        }
    }

    /// Insert newline - port of Go's InsertNewline()
    pub fn insert_newline(&mut self) {
        if self.max_height > 0 && self.value.len() >= self.max_height {
            return;
        }

        self.col = clamp(
            self.col,
            0,
            self.value.get(self.row).map_or(0, |line| line.len()),
        );
        self.split_line(self.row, self.col);
    }

    /// Move cursor one character left - port of Go's characterLeft()
    pub fn character_left(&mut self, inside_line: bool) {
        if self.col == 0 && self.row != 0 {
            self.row -= 1;
            if let Some(line) = self.value.get(self.row) {
                self.col = line.len();
                if !inside_line {
                    return;
                }
            }
        }
        if self.col > 0 {
            self.set_cursor(self.col - 1);
        }
    }

    /// Move cursor one character right - port of Go's characterRight()
    pub fn character_right(&mut self) {
        if let Some(line) = self.value.get(self.row) {
            if self.col < line.len() {
                self.set_cursor(self.col + 1);
            } else if self.row < self.value.len() - 1 {
                self.row += 1;
                self.cursor_start();
            }
        }
    }

    /// Move cursor one word left - port of Go's wordLeft()
    pub fn word_left(&mut self) {
        // Move left over any spaces
        while self.col > 0 {
            if let Some(line) = self.value.get(self.row) {
                if line.get(self.col - 1).is_some_and(|c| c.is_whitespace()) {
                    self.set_cursor(self.col - 1);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        // Then move left over the previous word
        while self.col > 0 {
            if let Some(line) = self.value.get(self.row) {
                if line.get(self.col - 1).is_some_and(|c| !c.is_whitespace()) {
                    self.set_cursor(self.col - 1);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Move cursor one word right - port of Go's wordRight()
    pub fn word_right(&mut self) {
        self.do_word_right(|_, _| {});
    }

    /// Internal word right with callback - port of Go's doWordRight()  
    fn do_word_right<F>(&mut self, mut func: F)
    where
        F: FnMut(usize, usize),
    {
        if self.row >= self.value.len() {
            return;
        }

        let line = match self.value.get(self.row) {
            Some(line) => line.clone(),
            None => return,
        };

        if self.col >= line.len() {
            return;
        }

        let mut pos = self.col;
        let mut char_idx = 0;

        // Skip any spaces at current position first
        while pos < line.len() && line[pos].is_whitespace() {
            pos += 1;
        }

        // Move through word characters until we reach whitespace or end
        while pos < line.len() && !line[pos].is_whitespace() {
            func(char_idx, pos);
            pos += 1;
            char_idx += 1;
        }

        // Update cursor position
        self.set_cursor(pos);
    }

    /// Transform word to uppercase - port of Go's uppercaseRight()
    pub fn uppercase_right(&mut self) {
        let start_col = self.col;
        let start_row = self.row;

        // Find the word boundaries
        self.word_right(); // Move to end of word
        let end_col = self.col;

        // Transform characters
        if let Some(line) = self.value.get_mut(start_row) {
            let end_idx = end_col.min(line.len());
            if let Some(slice) = line.get_mut(start_col..end_idx) {
                for ch in slice.iter_mut() {
                    *ch = ch.to_uppercase().next().unwrap_or(*ch);
                }
            }
        }
    }

    /// Transform word to lowercase - port of Go's lowercaseRight()  
    pub fn lowercase_right(&mut self) {
        let start_col = self.col;
        let start_row = self.row;

        // Find the word boundaries
        self.word_right(); // Move to end of word
        let end_col = self.col;

        // Transform characters
        if let Some(line) = self.value.get_mut(start_row) {
            let end_idx = end_col.min(line.len());
            if let Some(slice) = line.get_mut(start_col..end_idx) {
                for ch in slice.iter_mut() {
                    *ch = ch.to_lowercase().next().unwrap_or(*ch);
                }
            }
        }
    }

    /// Transform word to title case - port of Go's capitalizeRight()
    pub fn capitalize_right(&mut self) {
        let start_col = self.col;
        let start_row = self.row;

        // Find the word boundaries
        self.word_right(); // Move to end of word
        let end_col = self.col;

        // Transform characters
        if let Some(line) = self.value.get_mut(start_row) {
            let end_idx = end_col.min(line.len());
            if let Some(slice) = line.get_mut(start_col..end_idx) {
                for (i, ch) in slice.iter_mut().enumerate() {
                    if i == 0 {
                        *ch = ch.to_uppercase().next().unwrap_or(*ch);
                    }
                }
            }
        }
    }

    /// Transpose characters - port of Go's transposeLeft()
    pub fn transpose_left(&mut self) {
        let row = self.row;
        let mut col = self.col;

        if let Some(line) = self.value.get_mut(row) {
            if col == 0 || line.len() < 2 {
                return;
            }

            if col >= line.len() {
                col -= 1;
                self.col = col;
            }

            if col > 0 && col < line.len() {
                line.swap(col - 1, col);
                if col < line.len() {
                    self.col = col + 1;
                }
            }
        }
    }

    /// View renders the text area - port of Go's View()
    pub fn view(&mut self) -> String {
        // Early return for empty placeholder case
        if self.value.is_empty() || (self.value.len() == 1 && self.value[0].is_empty()) {
            return self.placeholder_view();
        }

        // Set cursor text style for rendering
        self.cursor.text_style = self.current_style.computed_cursor_line();

        let mut s = String::new();
        let line_info = self.line_info();
        let style = &self.current_style;

        // Track display lines and widest line number for padding
        let mut display_line = 0;
        let mut widest_line_number = 0;

        // Process each document line
        for (doc_line_idx, line) in self.value.iter().enumerate() {
            let wrapped_lines = self.cache.wrap(line, self.width);
            let is_current_doc_line = doc_line_idx == self.row;

            for (wrap_idx, wrapped_line) in wrapped_lines.iter().enumerate() {
                let prompt = self.get_prompt_string(display_line);
                s.push_str(&style.computed_prompt().render(&prompt));
                display_line += 1;

                // Line numbers
                let mut ln = String::new();
                if self.show_line_numbers {
                    if wrap_idx == 0 {
                        if is_current_doc_line {
                            ln = style
                                .computed_cursor_line_number()
                                .render(&self.format_line_number(doc_line_idx + 1));
                        } else {
                            ln = style
                                .computed_line_number()
                                .render(&self.format_line_number(doc_line_idx + 1));
                        }
                    } else if is_current_doc_line {
                        ln = style
                            .computed_cursor_line_number()
                            .render(&self.format_line_number(""));
                    } else {
                        ln = style
                            .computed_line_number()
                            .render(&self.format_line_number(""));
                    }
                    s.push_str(&ln);
                }

                // Track widest line number for padding
                let lnw = lipgloss::width(&ln);
                if lnw > widest_line_number {
                    widest_line_number = lnw;
                }

                let strwidth = wrapped_line
                    .iter()
                    .map(|&ch| unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0))
                    .sum::<usize>();
                let mut padding = self.width.saturating_sub(strwidth);

                // Handle width overflow from trailing spaces
                if strwidth > self.width {
                    // Remove trailing space if it causes overflow
                    let content: String = wrapped_line
                        .iter()
                        .collect::<String>()
                        .trim_end()
                        .to_string();
                    let new_wrapped_line: Vec<char> = content.chars().collect();
                    let new_strwidth = new_wrapped_line
                        .iter()
                        .map(|&ch| unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0))
                        .sum::<usize>();
                    padding = self.width.saturating_sub(new_strwidth);
                }

                // Render cursor if on current line and wrap
                if is_current_doc_line && line_info.row_offset == wrap_idx {
                    let col_offset = line_info.column_offset;

                    // Before cursor
                    let before: String = wrapped_line.iter().take(col_offset).collect();
                    s.push_str(&style.computed_cursor_line().render(&before));

                    // Cursor
                    if self.col >= line.len() && line_info.char_offset >= self.width {
                        self.cursor.set_char(" ");
                        s.push_str(&self.cursor.view());
                    } else {
                        let cursor_char = wrapped_line.get(col_offset).unwrap_or(&' ');
                        self.cursor.set_char(&cursor_char.to_string());
                        s.push_str(&self.cursor.view());

                        // After cursor
                        let after: String = wrapped_line.iter().skip(col_offset + 1).collect();
                        s.push_str(&style.computed_cursor_line().render(&after));
                    }
                } else {
                    // Regular line content
                    let content: String = wrapped_line.iter().collect();
                    let line_style = if is_current_doc_line {
                        style.computed_cursor_line()
                    } else {
                        style.computed_text()
                    };
                    s.push_str(&line_style.render(&content));
                }

                // Add padding
                s.push_str(&style.computed_text().render(&" ".repeat(padding.max(0))));
                s.push('\n');
            }
        }

        // Fill remaining height
        for _ in 0..(self.height.saturating_sub(display_line)) {
            let prompt = self.get_prompt_string(display_line);
            s.push_str(&style.computed_prompt().render(&prompt));
            display_line += 1;

            let left_gutter = self.end_of_buffer_character.to_string();
            let right_gap_width =
                self.width().saturating_sub(lipgloss::width(&left_gutter)) + widest_line_number;
            let right_gap = " ".repeat(right_gap_width.max(0));
            s.push_str(
                &style
                    .computed_end_of_buffer()
                    .render(&(left_gutter + &right_gap)),
            );
            s.push('\n');
        }

        // Strip ANSI codes for consistent test output
        lipgloss::strip_ansi(&s)
    }

    /// Get prompt string for a given display line - port of Go's getPromptString()
    fn get_prompt_string(&self, display_line: usize) -> String {
        if let Some(prompt_func) = self.prompt_func {
            let prompt = prompt_func(display_line);
            let pl = prompt.len();
            if pl < self.prompt_width {
                format!("{}{}", " ".repeat(self.prompt_width - pl), prompt)
            } else {
                prompt
            }
        } else {
            self.prompt.clone()
        }
    }

    /// Format line number for display - port of Go's formatLineNumber()
    fn format_line_number(&self, x: impl std::fmt::Display) -> String {
        // Calculate digits based on max height to ensure consistent formatting
        let digits = if self.max_height > 0 {
            self.max_height.to_string().len()
        } else {
            3 // Default to 3 digits as in Go implementation
        };
        format!(" {:width$} ", x, width = digits)
    }

    /// Render placeholder text - port of Go's placeholder view logic
    fn placeholder_view(&mut self) -> String {
        if self.placeholder.is_empty() {
            return String::new();
        }

        let mut s = String::new();

        // Split placeholder into lines
        let placeholder_lines: Vec<&str> = self.placeholder.lines().collect();

        for i in 0..self.height {
            // Render prompt
            let prompt = self.get_prompt_string(i);
            s.push_str(&prompt);

            // Render line numbers
            if self.show_line_numbers {
                let ln = if i == 0 {
                    self.format_line_number(1)
                } else {
                    self.format_line_number("")
                };
                s.push_str(&ln);
            }

            // Render line content
            if i < placeholder_lines.len() {
                // Render placeholder line (format_line_number already includes spacing)
                s.push_str(placeholder_lines[i]);
            } else {
                // End of buffer character for empty lines
                if self.end_of_buffer_character != ' ' {
                    s.push(self.end_of_buffer_character);
                }
            }

            s.push('\n');
        }

        // Remove trailing newline to match expected format
        s.trim_end_matches('\n').to_string()
    }

    /// Scroll viewport down by n lines - for testing viewport functionality
    pub fn scroll_down(&mut self, lines: usize) {
        self.viewport.set_y_offset(self.viewport.y_offset + lines);
    }

    /// Scroll viewport up by n lines - for testing viewport functionality  
    pub fn scroll_up(&mut self, lines: usize) {
        self.viewport
            .set_y_offset(self.viewport.y_offset.saturating_sub(lines));
    }

    /// Get cursor line number for display - port of Go's cursorLineNumber()
    pub fn cursor_line_number(&mut self) -> usize {
        if self.row >= self.value.len() {
            return 0;
        }

        // Count visual lines from all preceding document lines
        let mut line_count = 0;
        for i in 0..self.row {
            if let Some(line) = self.value.get(i).cloned() {
                let wrapped_lines = self.cache.wrap(&line, self.width);
                line_count += wrapped_lines.len();
            }
        }

        // Add the row offset within the current document line
        line_count += self.line_info().row_offset;
        line_count
    }

    /// Reposition the viewport to keep the cursor in view - port of Go's repositionView()
    fn reposition_view(&mut self) {
        let cursor_line = self.cursor_line_number();
        let minimum = self.viewport.y_offset;
        let maximum = minimum + self.viewport.height.saturating_sub(1);

        if cursor_line < minimum {
            // Cursor is above the visible area, scroll up
            self.viewport.set_y_offset(cursor_line);
        } else if cursor_line > maximum {
            // Cursor is below the visible area, scroll down
            let new_offset = cursor_line.saturating_sub(self.viewport.height.saturating_sub(1));
            self.viewport.set_y_offset(new_offset);
        }
    }

    /// Update handles incoming messages and updates the textarea state - port of Go's Update()
    pub fn update(&mut self, msg: Option<bubbletea_rs::Msg>) -> Option<bubbletea_rs::Cmd> {
        if !self.focus {
            return None;
        }

        if let Some(msg) = msg {
            // Handle clipboard messages first
            if let Some(paste_msg) = msg.downcast_ref::<PasteMsg>() {
                self.insert_string(paste_msg.0.clone());
                return None;
            }

            if let Some(_paste_err) = msg.downcast_ref::<PasteErrMsg>() {
                // Handle paste error - could be logged or shown to user
                return None;
            }

            // Handle key messages
            if let Some(key_msg) = msg.downcast_ref::<bubbletea_rs::KeyMsg>() {
                return self.handle_key_msg(key_msg);
            }

            // Pass other messages to cursor and viewport
            let cursor_cmd = self.cursor.update(&msg);
            let viewport_cmd = self.viewport.update(msg);

            // Return the first available command
            cursor_cmd.or(viewport_cmd)
        } else {
            None
        }
    }

    /// Handle key messages - port of Go's key handling logic
    fn handle_key_msg(&mut self, key_msg: &bubbletea_rs::KeyMsg) -> Option<bubbletea_rs::Cmd> {
        use crate::key::matches_binding;

        // Store old cursor position to determine if cursor moved
        let old_row = self.row;
        let old_col = self.col;

        // Character movement
        if matches_binding(key_msg, &self.key_map.character_forward) {
            self.character_right();
        } else if matches_binding(key_msg, &self.key_map.character_backward) {
            self.character_left(false);

        // Word movement
        } else if matches_binding(key_msg, &self.key_map.word_forward) {
            self.word_right();
        } else if matches_binding(key_msg, &self.key_map.word_backward) {
            self.word_left();

        // Line movement
        } else if matches_binding(key_msg, &self.key_map.line_next) {
            self.cursor_down();
        } else if matches_binding(key_msg, &self.key_map.line_previous) {
            self.cursor_up();
        } else if matches_binding(key_msg, &self.key_map.line_start) {
            self.cursor_start();
        } else if matches_binding(key_msg, &self.key_map.line_end) {
            self.cursor_end();

        // Input navigation
        } else if matches_binding(key_msg, &self.key_map.input_begin) {
            self.move_to_begin();
        } else if matches_binding(key_msg, &self.key_map.input_end) {
            self.move_to_end();

        // Deletion
        } else if matches_binding(key_msg, &self.key_map.delete_character_backward) {
            self.delete_character_backward();
        } else if matches_binding(key_msg, &self.key_map.delete_character_forward) {
            self.delete_character_forward();
        } else if matches_binding(key_msg, &self.key_map.delete_word_backward) {
            self.delete_word_backward();
        } else if matches_binding(key_msg, &self.key_map.delete_word_forward) {
            self.delete_word_forward();
        } else if matches_binding(key_msg, &self.key_map.delete_after_cursor) {
            self.delete_after_cursor();
        } else if matches_binding(key_msg, &self.key_map.delete_before_cursor) {
            self.delete_before_cursor();

        // Text insertion
        } else if matches_binding(key_msg, &self.key_map.insert_newline) {
            self.insert_newline();

        // Clipboard operations
        } else if matches_binding(key_msg, &self.key_map.paste) {
            return Some(self.paste_command());

        // Advanced text operations
        } else if matches_binding(key_msg, &self.key_map.uppercase_word_forward) {
            self.uppercase_right();
        } else if matches_binding(key_msg, &self.key_map.lowercase_word_forward) {
            self.lowercase_right();
        } else if matches_binding(key_msg, &self.key_map.capitalize_word_forward) {
            self.capitalize_right();
        } else if matches_binding(key_msg, &self.key_map.transpose_character_backward) {
            self.transpose_left();
        } else {
            // Handle regular character input
            if let Some(ch) = self.extract_character_from_key_msg(key_msg) {
                if ch.is_control() {
                    // Ignore control characters that aren't handled above
                    return None;
                }
                self.insert_rune(ch);
            }
        }

        // Reposition viewport if cursor moved or content changed
        if self.row != old_row || self.col != old_col {
            self.reposition_view();
        }

        None
    }

    /// Extract character from key message for regular text input
    fn extract_character_from_key_msg(&self, key_msg: &bubbletea_rs::KeyMsg) -> Option<char> {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Only extract printable characters without control modifiers
        // Skip if Control or Alt modifiers are pressed (these are for shortcuts)
        if key_msg.modifiers.contains(KeyModifiers::CONTROL)
            || key_msg.modifiers.contains(KeyModifiers::ALT)
        {
            return None;
        }

        match key_msg.key {
            KeyCode::Char(c) => {
                // Accept all printable characters
                if c.is_control() {
                    None
                } else {
                    Some(c)
                }
            }
            KeyCode::Tab => Some('\t'),
            _ => None,
        }
    }

    /// Create paste command for clipboard integration - port of Go's Paste()
    fn paste_command(&self) -> bubbletea_rs::Cmd {
        // Use tick with minimal duration to trigger clipboard read
        bubbletea_rs::tick(
            std::time::Duration::from_millis(1),
            |_| match Self::read_clipboard() {
                Ok(content) => Box::new(PasteMsg(content)) as bubbletea_rs::Msg,
                Err(err) => Box::new(PasteErrMsg(err)) as bubbletea_rs::Msg,
            },
        )
    }

    /// Read from system clipboard - matches textinput implementation
    fn read_clipboard() -> Result<String, String> {
        #[cfg(feature = "clipboard-support")]
        {
            use clipboard::{ClipboardContext, ClipboardProvider};

            let res: Result<String, String> = (|| {
                let mut ctx: ClipboardContext = ClipboardProvider::new()
                    .map_err(|e| format!("Failed to create clipboard context: {}", e))?;
                ctx.get_contents()
                    .map_err(|e| format!("Failed to read clipboard: {}", e))
            })();
            res
        }
        #[cfg(not(feature = "clipboard-support"))]
        {
            // Return empty string instead of error to avoid disrupting workflow
            Ok(String::new())
        }
    }

    /// Copy text to system clipboard
    pub fn copy_to_clipboard(&self, text: &str) -> Result<(), String> {
        #[cfg(feature = "clipboard-support")]
        {
            use clipboard::{ClipboardContext, ClipboardProvider};

            let mut ctx: ClipboardContext = ClipboardProvider::new()
                .map_err(|e| format!("Failed to create clipboard context: {}", e))?;

            ctx.set_contents(text.to_string())
                .map_err(|e| format!("Failed to write to clipboard: {}", e))
        }
        #[cfg(not(feature = "clipboard-support"))]
        {
            let _ = text; // Suppress unused parameter warning
            Err("Clipboard support not enabled".to_string())
        }
    }

    /// Copy current selection to clipboard (if selection is implemented)
    pub fn copy_selection(&self) -> Result<(), String> {
        // For now, copy entire content
        // In a full implementation, this would copy only selected text
        let content = self.value();
        self.copy_to_clipboard(&content)
    }

    /// Cut current selection to clipboard (if selection is implemented)
    pub fn cut_selection(&mut self) -> Result<(), String> {
        // For now, cut entire content
        // In a full implementation, this would cut only selected text
        let content = self.value();
        self.copy_to_clipboard(&content)?;
        self.reset();
        Ok(())
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

/// Focus sets the focus state on the model - port of Go's Focus()
impl Component for Model {
    fn focus(&mut self) -> Option<Cmd> {
        self.focus = true;
        self.current_style = self.focused_style.clone();
        self.cursor.focus()
    }

    fn blur(&mut self) {
        self.focus = false;
        self.current_style = self.blurred_style.clone();
        self.cursor.blur();
    }

    fn focused(&self) -> bool {
        self.focus
    }
}

/// Default styles matching Go's DefaultStyles() function
pub fn default_styles() -> (TextareaStyle, TextareaStyle) {
    let focused = default_focused_style();
    let blurred = default_blurred_style();
    (focused, blurred)
}

/// Create a new textarea model - convenience function
pub fn new() -> Model {
    Model::new()
}
