use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct Binding {
    pub keys: Vec<KeyCode>,
    pub help: String,
    pub description: String,
}

impl Binding {
    pub fn new(keys: Vec<KeyCode>) -> Self {
        Self {
            keys,
            help: String::new(),
            description: String::new(),
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = help.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn matches(&self, key_event: &KeyEvent) -> bool {
        self.keys.contains(&key_event.code)
    }
}

#[derive(Debug, Clone, Default)]
pub struct KeyMap {
    bindings: Vec<Binding>,
}

impl KeyMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_binding(mut self, binding: Binding) -> Self {
        self.bindings.push(binding);
        self
    }

    pub fn get_bindings(&self) -> &[Binding] {
        &self.bindings
    }

    pub fn find_binding(&self, key_event: &KeyEvent) -> Option<&Binding> {
        self.bindings.iter().find(|binding| binding.matches(key_event))
    }
}