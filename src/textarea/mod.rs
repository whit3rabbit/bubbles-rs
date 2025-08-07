pub mod memoization;

use crate::{cursor::Model as Cursor, viewport::Model as Viewport};

#[derive(Debug, Clone)]
pub struct Model {
    lines: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    viewport: Viewport,
    cursor: Cursor,
    width: usize,
    height: usize,
    wrap: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            viewport: Viewport::new(),
            cursor: Cursor::new(),
            width: 80,
            height: 24,
            wrap: true,
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dimensions(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        let value = value.into();
        self.lines = if value.is_empty() {
            vec![String::new()]
        } else {
            value.lines().map(|s| s.to_string()).collect()
        };
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    pub fn value(&self) -> String {
        self.lines.join("\n")
    }

    pub fn insert_char(&mut self, ch: char) {
        if ch == '\n' {
            self.insert_newline();
        } else {
            let line = &mut self.lines[self.cursor_row];
            line.insert(self.cursor_col, ch);
            self.cursor_col += 1;
        }
    }

    pub fn insert_newline(&mut self) {
        let current_line = &self.lines[self.cursor_row];
        let (left, right) = current_line.split_at(self.cursor_col);
        
        self.lines[self.cursor_row] = left.to_string();
        self.lines.insert(self.cursor_row + 1, right.to_string());
        
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    pub fn delete_char_backward(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            line.remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current_line);
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let line_len = self.lines[self.cursor_row].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            let line_len = self.lines[self.cursor_row].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    pub fn move_cursor_right(&mut self) {
        let line_len = self.lines[self.cursor_row].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn view(&self) -> String {
        let mut output = String::new();
        
        for (row_index, line) in self.lines.iter().enumerate() {
            if row_index == self.cursor_row {
                let (before, after) = line.split_at(self.cursor_col);
                output.push_str(before);
                output.push_str(&self.cursor.view());
                output.push_str(after);
            } else {
                output.push_str(line);
            }
            output.push('\n');
        }
        
        output
    }
}