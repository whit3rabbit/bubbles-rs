use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Model {
    current_directory: PathBuf,
    selected_file: Option<PathBuf>,
    show_hidden: bool,
    files: Vec<PathBuf>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            current_directory: std::env::current_dir().unwrap_or_default(),
            selected_file: None,
            show_hidden: false,
            files: Vec::new(),
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view(&self) -> String {
        format!("FilePicker: {:?}", self.current_directory)
    }
}