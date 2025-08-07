use crate::cursor::Model as Cursor;

#[derive(Debug, Clone)]
pub struct Model {
    value: String,
    cursor_position: usize,
    cursor: Cursor,
    placeholder: String,
    width: usize,
    focus: bool,
    echo_mode: EchoMode,
    char_limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum EchoMode {
    Normal,
    Password,
    None,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            value: String::new(),
            cursor_position: 0,
            cursor: Cursor::new(),
            placeholder: String::new(),
            width: 20,
            focus: false,
            echo_mode: EchoMode::Normal,
            char_limit: None,
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn with_echo_mode(mut self, mode: EchoMode) -> Self {
        self.echo_mode = mode;
        self
    }

    pub fn with_char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        if let Some(limit) = self.char_limit {
            if self.value.len() > limit {
                self.value.truncate(limit);
            }
        }
        self.cursor_position = self.cursor_position.min(self.value.len());
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn focus(&mut self) {
        self.focus = true;
        self.cursor.set_visible(true);
    }

    pub fn blur(&mut self) {
        self.focus = false;
        self.cursor.set_visible(false);
    }

    pub fn is_focused(&self) -> bool {
        self.focus
    }

    pub fn insert_char(&mut self, ch: char) {
        if let Some(limit) = self.char_limit {
            if self.value.len() >= limit {
                return;
            }
        }

        self.value.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    pub fn delete_char_backward(&mut self) {
        if self.cursor_position > 0 {
            self.value.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.value.len();
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.len() {
            self.cursor_position += 1;
        }
    }

    pub fn view(&self) -> String {
        let display_value = if self.value.is_empty() && !self.placeholder.is_empty() {
            &self.placeholder
        } else {
            match self.echo_mode {
                EchoMode::Normal => &self.value,
                EchoMode::Password => &"*".repeat(self.value.len()),
                EchoMode::None => "",
            }
        };

        let mut output = String::new();
        
        if self.focus && self.cursor_position <= display_value.len() {
            let (before, after) = display_value.split_at(self.cursor_position);
            output.push_str(before);
            output.push_str(&self.cursor.view());
            output.push_str(after);
        } else {
            output.push_str(display_value);
        }

        // Pad or truncate to width
        if output.len() < self.width {
            output.push_str(&" ".repeat(self.width - output.len()));
        } else if output.len() > self.width {
            output.truncate(self.width);
        }

        output
    }
}