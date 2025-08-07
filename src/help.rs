use crate::key::KeyMap;

#[derive(Debug, Clone)]
pub struct Model {
    show_all: bool,
    width: usize,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            show_all: false,
            width: 80,
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn view(&self, keymap: &KeyMap) -> String {
        "Help view placeholder".to_string()
    }
}