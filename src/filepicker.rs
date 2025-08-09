//! File picker component for browsing and selecting files in terminal applications.
//!
//! This module provides a fully-functional file picker that allows users to navigate
//! the file system, browse directories, and select files using keyboard navigation.
//! It integrates seamlessly with bubbletea-rs applications and supports customizable
//! styling and key bindings.
//!
//! # Features
//!
//! - **Directory Navigation**: Browse directories with vim-style key bindings
//! - **File Selection**: Select files using Enter key
//! - **Cross-platform**: Works on Windows, macOS, and Linux
//! - **Hidden File Handling**: Automatically hides dotfiles and system hidden files
//! - **Customizable Styling**: Configurable colors and styles for different file types
//! - **Keyboard Navigation**: Full keyboard support with configurable key bindings
//! - **Sorting**: Directories are automatically sorted before files alphabetically
//!
//! # Basic Usage
//!
//! ```rust
//! use bubbletea_widgets::filepicker::Model;
//! use bubbletea_rs::{Model as BubbleTeaModel, Msg, Cmd};
//!
//! // Create a new file picker
//! let (mut filepicker, cmd) = Model::init();
//!
//! // In your application's update method
//! fn handle_filepicker_msg(filepicker: &mut Model, msg: Msg) -> Option<Cmd> {
//!     // Check if a file was selected
//!     let (selected, path) = filepicker.did_select_file(&msg);
//!     if selected {
//!         if let Some(file_path) = path {
//!             println!("Selected file: {:?}", file_path);
//!         }
//!     }
//!     
//!     // Update the filepicker
//!     filepicker.update(msg)
//! }
//! ```
//!
//! # Customization
//!
//! ```rust
//! use bubbletea_widgets::filepicker::{Model, Styles, FilepickerKeyMap};
//! use lipgloss::{Style, Color};
//!
//! let mut filepicker = Model::new();
//!
//! // Customize styles
//! filepicker.styles.cursor = Style::new().foreground(Color::from("cyan"));
//! filepicker.styles.directory = Style::new().foreground(Color::from("blue")).bold(true);
//! filepicker.styles.file = Style::new().foreground(Color::from("white"));
//!
//! // The keymap can also be customized if needed
//! // filepicker.keymap = FilepickerKeyMap::default();
//! ```
//!
//! # Key Bindings
//!
//! The default key bindings are:
//!
//! - `j`/`↓`: Move cursor down
//! - `k`/`↑`: Move cursor up  
//! - `l`/`→`/`Enter`: Open directory or select file
//! - `h`/`←`/`Backspace`/`Esc`: Go back to parent directory
//! - `PageUp`/`b`: Page up
//! - `PageDown`/`f`: Page down

use crate::key::{self, KeyMap};
use bubbletea_rs::{Cmd, KeyMsg, Model as BubbleTeaModel, Msg};
use lipgloss_extras::prelude::*;
use std::path::{Path, PathBuf};

/// Key bindings for filepicker navigation and interaction.
///
/// This struct defines all the keyboard shortcuts available in the file picker.
/// Each binding supports multiple keys for the same action (e.g., both 'j' and '↓' for moving down).
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::filepicker::FilepickerKeyMap;
/// use bubbletea_widgets::key::KeyMap;
///
/// let keymap = FilepickerKeyMap::default();
/// let help_keys = keymap.short_help(); // Get keys for help display
/// ```
#[derive(Debug, Clone)]
pub struct FilepickerKeyMap {
    /// Key binding for moving the cursor down in the file list.
    /// Default: 'j', '↓'
    pub down: key::Binding,
    /// Key binding for moving the cursor up in the file list.
    /// Default: 'k', '↑'
    pub up: key::Binding,
    /// Key binding for navigating back to the parent directory.
    /// Default: 'h', '←', 'Backspace', 'Esc'
    pub back: key::Binding,
    /// Key binding for opening directories or selecting files.
    /// Default: 'l', '→', 'Enter'
    pub open: key::Binding,
    /// Key binding for selecting the current file (alternative to open).
    /// Default: 'Enter'
    pub select: key::Binding,
    /// Key binding for scrolling up a page in the file list.
    /// Default: 'PageUp', 'b'
    pub page_up: key::Binding,
    /// Key binding for scrolling down a page in the file list.
    /// Default: 'PageDown', 'f'
    pub page_down: key::Binding,
}

impl Default for FilepickerKeyMap {
    fn default() -> Self {
        use crossterm::event::KeyCode;

        Self {
            down: key::Binding::new(vec![KeyCode::Char('j'), KeyCode::Down])
                .with_help("j/↓", "down"),
            up: key::Binding::new(vec![KeyCode::Char('k'), KeyCode::Up]).with_help("k/↑", "up"),
            back: key::Binding::new(vec![
                KeyCode::Char('h'),
                KeyCode::Backspace,
                KeyCode::Left,
                KeyCode::Esc,
            ])
            .with_help("h/←", "back"),
            open: key::Binding::new(vec![KeyCode::Char('l'), KeyCode::Right, KeyCode::Enter])
                .with_help("l/→", "open"),
            select: key::Binding::new(vec![KeyCode::Enter]).with_help("enter", "select"),
            page_up: key::Binding::new(vec![KeyCode::PageUp, KeyCode::Char('b')])
                .with_help("pgup/b", "page up"),
            page_down: key::Binding::new(vec![KeyCode::PageDown, KeyCode::Char('f')])
                .with_help("pgdn/f", "page down"),
        }
    }
}

impl KeyMap for FilepickerKeyMap {
    fn short_help(&self) -> Vec<&key::Binding> {
        vec![&self.up, &self.down, &self.open, &self.back]
    }

    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![
            vec![&self.up, &self.down],
            vec![&self.open, &self.back, &self.select],
            vec![&self.page_up, &self.page_down],
        ]
    }
}

/// Visual styling configuration for the file picker.
///
/// This struct allows customization of colors and styles for different elements
/// of the file picker interface, including the cursor, directories, files, and selected items.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::filepicker::Styles;
/// use lipgloss::{Style, Color};
///
/// let mut styles = Styles::default();
/// styles.cursor = Style::new().foreground(Color::from("cyan"));
/// styles.directory = Style::new().foreground(Color::from("blue")).bold(true);
/// ```
#[derive(Debug, Clone)]
pub struct Styles {
    /// Style for the cursor indicator (usually "> ").
    /// Default: foreground color 212 (pink/magenta)
    pub cursor: Style,
    /// Style for directory names in the file list.
    /// Default: foreground color 99 (purple)
    pub directory: Style,
    /// Style for regular file names in the file list.
    /// Default: no special styling (terminal default)
    pub file: Style,
    /// Style applied to the currently selected item (in addition to file/directory style).
    /// Default: foreground color 212 (pink/magenta) and bold
    pub selected: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            cursor: Style::new().foreground(Color::from("212")),
            directory: Style::new().foreground(Color::from("99")),
            file: Style::new(),
            selected: Style::new().foreground(Color::from("212")).bold(true),
        }
    }
}

/// Represents a single file or directory entry in the file picker.
///
/// This struct contains all the information needed to display and interact with
/// a file system entry, including its name, full path, and whether it's a directory.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::filepicker::FileEntry;
/// use std::path::PathBuf;
///
/// let entry = FileEntry {
///     name: "example.txt".to_string(),
///     path: PathBuf::from("/path/to/example.txt"),
///     is_dir: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// The display name of the file or directory.
    /// This is typically just the filename without the full path.
    pub name: String,
    /// The complete path to the file or directory.
    pub path: PathBuf,
    /// Whether this entry represents a directory (`true`) or a file (`false`).
    pub is_dir: bool,
}

/// The main file picker model containing all state and configuration.
///
/// This struct represents the complete state of the file picker, including the current
/// directory, file list, selection state, and styling configuration. It implements
/// the BubbleTeaModel trait for integration with bubbletea-rs applications.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::filepicker::Model;
/// use bubbletea_rs::Model as BubbleTeaModel;
///
/// // Create a new file picker
/// let mut picker = Model::new();
///
/// // Or use the BubbleTeaModel::init() method
/// let (picker, cmd) = Model::init();
/// ```
///
/// # State Management
///
/// The model maintains:
/// - Current directory being browsed
/// - List of files and directories in the current location
/// - Currently selected item index
/// - Last selected file path (if any)
/// - Styling and key binding configuration
#[derive(Debug, Clone)]
pub struct Model {
    /// The directory currently being browsed.
    /// This path is updated when navigating into subdirectories or back to parent directories.
    pub current_directory: PathBuf,
    /// Key bindings configuration for navigation and interaction.
    /// Can be customized to change keyboard shortcuts.
    pub keymap: FilepickerKeyMap,
    /// Visual styling configuration for different UI elements.
    /// Can be customized to change colors and appearance.
    pub styles: Styles,
    files: Vec<FileEntry>,
    selected: usize,
    /// The path of the most recently selected file, if any.
    /// This is set when a user selects a file (not a directory) and can be checked
    /// using the `did_select_file()` method.
    pub path: Option<PathBuf>,
}

impl Model {
    /// Creates a new file picker model with default settings.
    ///
    /// The file picker starts in the current working directory (".") and uses
    /// default key bindings and styles. The file list is initially empty and will
    /// be populated when the model is initialized or when directories are navigated.
    ///
    /// # Returns
    ///
    /// A new `Model` instance with:
    /// - Current directory set to "."
    /// - Default key bindings
    /// - Default styling
    /// - Empty file list (call `read_dir()` or use `BubbleTeaModel::init()` to populate)
    /// - No file selected
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::filepicker::Model;
    ///
    /// let mut picker = Model::new();
    /// assert_eq!(picker.current_directory.as_os_str(), ".");
    /// assert!(picker.path.is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            current_directory: PathBuf::from("."),
            keymap: FilepickerKeyMap::default(),
            styles: Styles::default(),
            files: vec![],
            selected: 0,
            path: None,
        }
    }

    /// Checks if a file was selected in response to the given message.
    ///
    /// This method examines the provided message to determine if it represents
    /// a file selection event. It returns both a boolean indicating whether a
    /// file was selected and the path of the selected file (if any).
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to check for file selection events
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `bool`: `true` if a file was selected, `false` otherwise
    /// - `Option<PathBuf>`: The path of the selected file, or `None` if no file was selected
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::filepicker::Model;
    /// use bubbletea_rs::{KeyMsg, Msg};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let mut picker = Model::new();
    ///
    /// // Simulate an Enter key press
    /// let key_msg = KeyMsg {
    ///     key: KeyCode::Enter,
    ///     modifiers: KeyModifiers::NONE,
    /// };
    /// let msg: Msg = Box::new(key_msg);
    ///
    /// let (selected, path) = picker.did_select_file(&msg);
    /// if selected {
    ///     println!("File selected: {:?}", path);
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// This method only returns `true` if:
    /// 1. The message is a key press of the Enter key
    /// 2. A file path is currently stored in `self.path` (i.e., a file was previously highlighted)
    pub fn did_select_file(&self, msg: &Msg) -> (bool, Option<PathBuf>) {
        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            if key_msg.key == crossterm::event::KeyCode::Enter && self.path.is_some() {
                return (true, self.path.clone());
            }
        }
        (false, None)
    }

    fn read_dir(&mut self) {
        self.files.clear();
        if let Ok(entries) = std::fs::read_dir(&self.current_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();

                // Skip hidden files (platform-aware)
                if is_hidden(&path, &name) {
                    continue;
                }

                let is_dir = path.is_dir();
                self.files.push(FileEntry { name, path, is_dir });
            }
        }

        // Sort directories first, then files
        self.files
            .sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));

        self.selected = 0;
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

/// Determines whether a file or directory should be considered hidden.
///
/// This function implements cross-platform hidden file detection:
/// - On Windows: Checks the FILE_ATTRIBUTE_HIDDEN attribute
/// - On Unix-like systems: Files/directories starting with '.' are considered hidden
///
/// # Arguments
///
/// * `path` - The full path to the file or directory
/// * `name` - The filename/directory name (used as fallback on Windows)
///
/// # Returns
///
/// `true` if the file should be hidden, `false` otherwise
#[inline]
fn is_hidden(path: &Path, name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(meta) = path.metadata() {
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            return (meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0;
        }
        // Fallback: also consider dotfiles as hidden if attribute lookup fails
        return name.starts_with('.');
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path; // silence unused on non-Windows
        name.starts_with('.')
    }
}

impl BubbleTeaModel for Model {
    fn init() -> (Self, Option<Cmd>) {
        let mut model = Self::new();
        model.read_dir();
        (model, None)
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            if self.keymap.down.matches(key_msg) {
                if self.selected < self.files.len().saturating_sub(1) {
                    self.selected += 1;
                }
            } else if self.keymap.up.matches(key_msg) {
                self.selected = self.selected.saturating_sub(1);
            } else if self.keymap.back.matches(key_msg) {
                self.current_directory.pop();
                self.read_dir();
            } else if self.keymap.open.matches(key_msg) && !self.files.is_empty() {
                let entry = &self.files[self.selected];
                if entry.is_dir {
                    self.current_directory = entry.path.clone();
                    self.read_dir();
                } else {
                    self.path = Some(entry.path.clone());
                }
            }
        }
        None
    }

    fn view(&self) -> String {
        if self.files.is_empty() {
            return "No files found.".to_string();
        }

        let mut output = String::new();
        for (i, entry) in self.files.iter().enumerate() {
            if i == self.selected {
                output.push_str(&self.styles.cursor.render("> "));
                let style = if entry.is_dir {
                    &self.styles.directory
                } else {
                    &self.styles.file
                };
                output.push_str(&self.styles.selected.render(&style.render(&entry.name)));
            } else {
                output.push_str("  ");
                let style = if entry.is_dir {
                    &self.styles.directory
                } else {
                    &self.styles.file
                };
                output.push_str(&style.render(&entry.name));
            }
            output.push('\n');
        }

        output
    }
}
