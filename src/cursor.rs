use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Model {
    blink: bool,
    visible: bool,
    char: char,
    text_style: String,
    focus_style: String,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            blink: true,
            visible: true,
            char: '|',
            text_style: String::new(),
            focus_style: String::new(),
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_blink(mut self, blink: bool) -> Self {
        self.blink = blink;
        self
    }

    pub fn with_char(mut self, char: char) -> Self {
        self.char = char;
        self
    }

    pub fn view(&self) -> String {
        if !self.visible {
            return " ".to_string();
        }
        self.char.to_string()
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn tick_interval() -> Duration {
        Duration::from_millis(500)
    }
}