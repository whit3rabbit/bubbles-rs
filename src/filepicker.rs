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
//! - **Hidden File Handling**: Cross-platform hidden file detection (Windows attributes + dotfiles on Unix)
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
//!         println!("Selected file: {:?}", path);
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
//! use lipgloss_extras::prelude::*;
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
use std::sync::atomic::{AtomicI64, Ordering};

/// Global counter for generating unique filepicker instance IDs.
static LAST_ID: AtomicI64 = AtomicI64::new(0);

/// Generates a unique ID for filepicker instances.
///
/// This function provides thread-safe ID generation for distinguishing
/// between multiple filepicker instances in the same application.
///
/// # Returns
///
/// A unique i64 identifier
fn next_id() -> i64 {
    LAST_ID.fetch_add(1, Ordering::SeqCst) + 1
}

/// Message type for handling errors during file system operations.
///
/// This message is sent when file system operations like reading directories fail.
/// It contains the error description that can be displayed to the user.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::filepicker::ErrorMsg;
///
/// let error_msg = ErrorMsg {
///     err: "Permission denied".to_string(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ErrorMsg {
    /// The error message string.
    pub err: String,
}

/// Message type for handling successful directory reads.
///
/// This message is sent when a directory is successfully read and contains
/// the file entries found in that directory. The ID is used to match responses
/// to the correct filepicker instance in applications with multiple pickers.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::filepicker::{ReadDirMsg, FileEntry};
/// use std::path::PathBuf;
///
/// let msg = ReadDirMsg {
///     id: 1,
///     entries: vec![
///         FileEntry {
///             name: "file.txt".to_string(),
///             path: PathBuf::from("./file.txt"),
///             is_dir: false,
///             is_symlink: false,
///             size: 1024,
///             mode: 0o644,
///             symlink_target: None,
///         }
///     ],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ReadDirMsg {
    /// The ID of the filepicker instance that requested this read.
    pub id: i64,
    /// The file entries found in the directory.
    pub entries: Vec<FileEntry>,
}

const MARGIN_BOTTOM: usize = 5;
const FILE_SIZE_WIDTH: usize = 7;

#[allow(dead_code)]
const PADDING_LEFT: usize = 2;

/// A simple stack implementation for managing navigation history.
///
/// This stack is used internally to remember cursor positions and viewport
/// state when navigating into subdirectories, allowing the filepicker to
/// restore the previous selection when navigating back.
///
/// # Examples
///
/// ```rust
/// // Note: This is a private struct, shown for documentation purposes
/// // let mut stack = Stack::new();
/// // stack.push(5);
/// // assert_eq!(stack.pop(), Some(5));
/// ```
#[derive(Debug, Clone, Default)]
struct Stack {
    items: Vec<usize>,
}

impl Stack {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn push(&mut self, item: usize) {
        self.items.push(item);
    }

    fn pop(&mut self) -> Option<usize> {
        self.items.pop()
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

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
    /// Key binding for jumping to the top of the file list.
    /// Default: 'g'
    pub go_to_top: key::Binding,
    /// Key binding for jumping to the last item in the file list.
    /// Default: 'G'
    pub go_to_last: key::Binding,
    /// Key binding for moving the cursor down in the file list.
    /// Default: 'j', '↓'
    pub down: key::Binding,
    /// Key binding for moving the cursor up in the file list.
    /// Default: 'k', '↑'
    pub up: key::Binding,
    /// Key binding for scrolling up a page in the file list.
    /// Default: 'PageUp', 'K'
    pub page_up: key::Binding,
    /// Key binding for scrolling down a page in the file list.
    /// Default: 'PageDown', 'J'
    pub page_down: key::Binding,
    /// Key binding for navigating back to the parent directory.
    /// Default: 'h', '←', 'Backspace', 'Esc'
    pub back: key::Binding,
    /// Key binding for opening directories or selecting files.
    /// Default: 'l', '→', 'Enter'
    pub open: key::Binding,
    /// Key binding for selecting the current file (alternative to open).
    /// Default: 'Enter'
    pub select: key::Binding,
}

impl Default for FilepickerKeyMap {
    fn default() -> Self {
        use crossterm::event::KeyCode;

        Self {
            go_to_top: key::Binding::new(vec![KeyCode::Char('g')]).with_help("g", "first"),
            go_to_last: key::Binding::new(vec![KeyCode::Char('G')]).with_help("G", "last"),
            down: key::Binding::new(vec![KeyCode::Char('j'), KeyCode::Down])
                .with_help("j/↓", "down"),
            up: key::Binding::new(vec![KeyCode::Char('k'), KeyCode::Up]).with_help("k/↑", "up"),
            page_up: key::Binding::new(vec![KeyCode::PageUp, KeyCode::Char('K')])
                .with_help("pgup/K", "page up"),
            page_down: key::Binding::new(vec![KeyCode::PageDown, KeyCode::Char('J')])
                .with_help("pgdn/J", "page down"),
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
        }
    }
}

impl KeyMap for FilepickerKeyMap {
    fn short_help(&self) -> Vec<&key::Binding> {
        vec![&self.up, &self.down, &self.open, &self.back]
    }

    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![
            vec![&self.go_to_top, &self.go_to_last],
            vec![&self.up, &self.down],
            vec![&self.page_up, &self.page_down],
            vec![&self.open, &self.back, &self.select],
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
/// use lipgloss_extras::prelude::*;
///
/// let mut styles = Styles::default();
/// styles.cursor = Style::new().foreground(Color::from("cyan"));
/// styles.directory = Style::new().foreground(Color::from("blue")).bold(true);
/// ```
#[derive(Debug, Clone)]
pub struct Styles {
    /// Style for the cursor when disabled.
    /// Default: foreground color 247 (gray)
    pub disabled_cursor: Style,
    /// Style for the cursor indicator (usually "> ").
    /// Default: foreground color 212 (pink/magenta)
    pub cursor: Style,
    /// Style for symlink file names.
    /// Default: foreground color 36 (cyan)
    pub symlink: Style,
    /// Style for directory names in the file list.
    /// Default: foreground color 99 (purple)
    pub directory: Style,
    /// Style for regular file names in the file list.
    /// Default: no special styling (terminal default)
    pub file: Style,
    /// Style for disabled/unselectable files.
    /// Default: foreground color 243 (dark gray)
    pub disabled_file: Style,
    /// Style for file permissions display.
    /// Default: foreground color 244 (gray)
    pub permission: Style,
    /// Style applied to the currently selected item (in addition to file/directory style).
    /// Default: foreground color 212 (pink/magenta) and bold
    pub selected: Style,
    /// Style for disabled selected items.
    /// Default: foreground color 247 (gray)
    pub disabled_selected: Style,
    /// Style for file size display.
    /// Default: foreground color 240 (dark gray), right-aligned
    pub file_size: Style,
    /// Style for empty directory message.
    /// Default: foreground color 240 (dark gray) with left padding
    pub empty_directory: Style,
}

impl Default for Styles {
    fn default() -> Self {
        const FILE_SIZE_WIDTH: usize = 7;
        const PADDING_LEFT: usize = 2;

        Self {
            disabled_cursor: Style::new().foreground(Color::from("247")),
            cursor: Style::new().foreground(Color::from("212")),
            symlink: Style::new().foreground(Color::from("36")),
            directory: Style::new().foreground(Color::from("99")),
            file: Style::new(),
            disabled_file: Style::new().foreground(Color::from("243")),
            permission: Style::new().foreground(Color::from("244")),
            selected: Style::new().foreground(Color::from("212")).bold(true),
            disabled_selected: Style::new().foreground(Color::from("247")),
            file_size: Style::new()
                .foreground(Color::from("240"))
                .width(FILE_SIZE_WIDTH as i32),
            empty_directory: Style::new()
                .foreground(Color::from("240"))
                .padding_left(PADDING_LEFT as i32),
        }
    }
}

/// Represents a single file or directory entry in the file picker.
///
/// This struct contains all the information needed to display and interact with
/// a file system entry, including its name, full path, metadata, and type information.
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
///     is_symlink: false,
///     size: 1024,
///     mode: 0o644,
///     symlink_target: None,
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
    /// Whether this entry is a symbolic link.
    pub is_symlink: bool,
    /// File size in bytes.
    pub size: u64,
    /// File permissions mode.
    pub mode: u32,
    /// Target path if this is a symlink.
    pub symlink_target: Option<PathBuf>,
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
/// - Navigation history and viewport state
#[derive(Debug, Clone)]
pub struct Model {
    id: i64,

    /// Path is the path which the user has selected with the file picker.
    pub path: String,

    /// The directory currently being browsed.
    /// This path is updated when navigating into subdirectories or back to parent directories.
    pub current_directory: PathBuf,

    /// AllowedTypes specifies which file types the user may select.
    /// If empty the user may select any file.
    pub allowed_types: Vec<String>,

    /// Key bindings configuration for navigation and interaction.
    /// Can be customized to change keyboard shortcuts.
    pub keymap: FilepickerKeyMap,

    files: Vec<FileEntry>,

    /// Whether to show file permissions in the display.
    pub show_permissions: bool,
    /// Whether to show file sizes in the display.
    pub show_size: bool,
    /// Whether to show hidden files (dotfiles on Unix, Windows FILE_ATTRIBUTE_HIDDEN + dotfiles).
    pub show_hidden: bool,
    /// Whether directories can be selected.
    pub dir_allowed: bool,
    /// Whether files can be selected.
    pub file_allowed: bool,

    /// The name of the most recently selected file.
    pub file_selected: String,

    selected: usize,
    selected_stack: Stack,

    min: usize,
    max: usize,
    max_stack: Stack,
    min_stack: Stack,

    /// Height of the picker.
    pub height: usize,
    /// Whether height should automatically adjust to terminal size.
    pub auto_height: bool,

    /// The cursor string to display (e.g., "> ").
    pub cursor: String,

    /// Error message to display when directory operations fail.
    pub error: Option<String>,

    /// Visual styling configuration for different UI elements.
    /// Can be customized to change colors and appearance.
    pub styles: Styles,
}

/// Creates a new filepicker model with default styling and key bindings.
///
/// This function provides a convenient way to create a filepicker without
/// having to call `Model::new()` directly. It matches the Go implementation's
/// `New()` function for API compatibility.
///
/// # Returns
///
/// A new `Model` instance with default settings, starting in the current directory.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::filepicker;
///
/// let picker = filepicker::new();
/// assert_eq!(picker.current_directory.as_os_str(), ".");
/// ```
pub fn new() -> Model {
    Model::new()
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
    /// A new `Model` instance with default settings matching the Go implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::filepicker::Model;
    ///
    /// let mut picker = Model::new();
    /// assert_eq!(picker.current_directory.as_os_str(), ".");
    /// assert!(picker.path.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            id: next_id(),
            path: String::new(),
            current_directory: PathBuf::from("."),
            allowed_types: Vec::new(),
            keymap: FilepickerKeyMap::default(),
            files: Vec::new(),
            show_permissions: true,
            show_size: true,
            show_hidden: false,
            dir_allowed: false,
            file_allowed: true,
            file_selected: String::new(),
            selected: 0,
            selected_stack: Stack::new(),
            min: 0,
            max: 0,
            max_stack: Stack::new(),
            min_stack: Stack::new(),
            height: 0,
            auto_height: true,
            cursor: ">".to_string(),
            error: None,
            styles: Styles::default(),
        }
    }

    /// Sets the height of the filepicker viewport.
    ///
    /// This controls how many file entries are visible at once. The viewport
    /// automatically adjusts to show the selected item within the visible range.
    ///
    /// # Arguments
    ///
    /// * `height` - The number of lines to show in the file list
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::filepicker::Model;
    ///
    /// let mut picker = Model::new();
    /// picker.set_height(10); // Show 10 files at a time
    /// ```
    pub fn set_height(&mut self, height: usize) {
        self.height = height;
        if self.max > self.height.saturating_sub(1) {
            self.max = self.min + self.height - 1;
        }
    }

    fn push_view(&mut self, selected: usize, minimum: usize, maximum: usize) {
        self.selected_stack.push(selected);
        self.min_stack.push(minimum);
        self.max_stack.push(maximum);
    }

    fn pop_view(&mut self) -> (usize, usize, usize) {
        let selected = self.selected_stack.pop().unwrap_or(0);
        let min = self.min_stack.pop().unwrap_or(0);
        let max = self.max_stack.pop().unwrap_or(0);
        (selected, min, max)
    }

    /// Returns whether a user has selected a file with the given message.
    ///
    /// This function checks if the message represents a file selection action
    /// and if the selected file is allowed based on the current configuration.
    /// It only returns `true` for files that can actually be selected.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to check for file selection
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `bool`: Whether a valid file was selected
    /// - `String`: The path of the selected file (empty if no selection)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bubbletea_widgets::filepicker::Model;
    /// use bubbletea_rs::Msg;
    ///
    /// let picker = Model::new();
    /// // In your application's update loop:
    /// // let (selected, path) = picker.did_select_file(&msg);
    /// // if selected {
    /// //     println!("User selected: {}", path);
    /// // }
    /// ```
    pub fn did_select_file(&self, msg: &Msg) -> (bool, String) {
        let (did_select, path) = self.did_select_file_internal(msg);
        if did_select && self.can_select(&path) {
            (true, path)
        } else {
            (false, String::new())
        }
    }

    /// Returns whether a user tried to select a disabled file with the given message.
    ///
    /// This function is useful for providing feedback when users attempt to select
    /// files that are not allowed based on the current `allowed_types` configuration.
    /// Use this to show warning messages or provide helpful feedback.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to check for disabled file selection attempts
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `bool`: Whether a disabled file selection was attempted
    /// - `String`: The path of the disabled file (empty if no disabled selection)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bubbletea_widgets::filepicker::Model;
    /// use bubbletea_rs::Msg;
    ///
    /// let mut picker = Model::new();
    /// picker.allowed_types = vec![".txt".to_string()];
    ///
    /// // In your application's update loop:
    /// // let (tried_disabled, path) = picker.did_select_disabled_file(&msg);
    /// // if tried_disabled {
    /// //     eprintln!("Cannot select {}: file type not allowed", path);
    /// // }
    /// ```
    pub fn did_select_disabled_file(&self, msg: &Msg) -> (bool, String) {
        let (did_select, path) = self.did_select_file_internal(msg);
        if did_select && !self.can_select(&path) {
            (true, path)
        } else {
            (false, String::new())
        }
    }

    fn did_select_file_internal(&self, msg: &Msg) -> (bool, String) {
        if self.files.is_empty() {
            return (false, String::new());
        }

        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            // If the msg does not match the Select keymap then this could not have been a selection.
            if !self.keymap.select.matches(key_msg) {
                return (false, String::new());
            }

            // The key press was a selection, let's confirm whether the current file could
            // be selected or used for navigating deeper into the stack.
            let f = &self.files[self.selected];
            let is_dir = f.is_dir;

            if (!is_dir && self.file_allowed)
                || (is_dir && self.dir_allowed) && !self.path.is_empty()
            {
                return (true, self.path.clone());
            }
        }
        (false, String::new())
    }

    fn can_select(&self, file: &str) -> bool {
        if self.allowed_types.is_empty() {
            return true;
        }

        for ext in &self.allowed_types {
            if file.ends_with(ext) {
                return true;
            }
        }
        false
    }

    /// Reads the current directory and populates the files list.
    /// Clears any existing files and error state before reading.
    pub fn read_dir(&mut self) {
        self.files.clear();
        self.error = None;
        match std::fs::read_dir(&self.current_directory) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                        .to_string();

                    // Skip hidden files if not showing them
                    if !self.show_hidden && is_hidden(&path, &name) {
                        continue;
                    }

                    // Get file metadata
                    let (is_dir, is_symlink, size, mode, symlink_target) =
                        if let Ok(metadata) = entry.metadata() {
                            let is_symlink = metadata.file_type().is_symlink();
                            let mut is_dir = metadata.is_dir();
                            let size = metadata.len();

                            #[cfg(unix)]
                            let mode = {
                                use std::os::unix::fs::PermissionsExt;
                                metadata.permissions().mode()
                            };
                            #[cfg(not(unix))]
                            let mode = 0;

                            // Handle symlink resolution
                            let symlink_target = if is_symlink {
                                match std::fs::canonicalize(&path) {
                                    Ok(target) => {
                                        // Check if symlink points to a directory
                                        if let Ok(target_meta) = std::fs::metadata(&target) {
                                            if target_meta.is_dir() {
                                                is_dir = true;
                                            }
                                        }
                                        Some(target)
                                    }
                                    Err(_) => None,
                                }
                            } else {
                                None
                            };

                            (is_dir, is_symlink, size, mode, symlink_target)
                        } else {
                            (path.is_dir(), false, 0, 0, None)
                        };

                    self.files.push(FileEntry {
                        name,
                        path,
                        is_dir,
                        is_symlink,
                        size,
                        mode,
                        symlink_target,
                    });
                }

                // Sort directories first, then files, then alphabetically
                self.files
                    .sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));

                self.selected = 0;
                self.max = std::cmp::max(self.max, self.height.saturating_sub(1));
            }
            Err(err) => {
                self.error = Some(format!("Failed to read directory: {}", err));
            }
        }
    }

    /// Creates a command to read the current directory.
    ///
    /// This method allows external code to trigger a directory read without
    /// directly calling the private `read_dir` method. It's useful for
    /// initializing the filepicker with a specific directory.
    ///
    /// # Returns
    ///
    /// A command that will trigger a ReadDirMsg when executed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::filepicker::Model;
    ///
    /// let mut picker = Model::new();
    /// picker.current_directory = std::path::PathBuf::from("/home/user");
    /// let cmd = picker.read_dir_cmd();
    /// // This command can be returned from init() or update()
    /// ```
    pub fn read_dir_cmd(&self) -> Cmd {
        // Use bubbletea_rs tick with minimal delay to create an immediate command
        let current_dir = self.current_directory.clone();
        let id = self.id;

        bubbletea_rs::tick(std::time::Duration::from_nanos(1), move |_| {
            let mut entries = Vec::new();

            if let Ok(dir_entries) = std::fs::read_dir(&current_dir) {
                for entry in dir_entries.flatten() {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                        .to_string();

                    // Skip hidden files by default (can be configured later)
                    if name.starts_with('.') {
                        continue;
                    }

                    // Get file metadata
                    let (is_dir, is_symlink, size, mode, symlink_target) =
                        if let Ok(metadata) = entry.metadata() {
                            let is_symlink = metadata.file_type().is_symlink();
                            let mut is_dir = metadata.is_dir();
                            let size = metadata.len();

                            #[cfg(unix)]
                            let mode = {
                                use std::os::unix::fs::PermissionsExt;
                                metadata.permissions().mode()
                            };
                            #[cfg(not(unix))]
                            let mode = 0;

                            // Handle symlink resolution
                            let symlink_target = if is_symlink {
                                match std::fs::canonicalize(&path) {
                                    Ok(target) => {
                                        // Check if symlink points to a directory
                                        if let Ok(target_meta) = std::fs::metadata(&target) {
                                            if target_meta.is_dir() {
                                                is_dir = true;
                                            }
                                        }
                                        Some(target)
                                    }
                                    Err(_) => None,
                                }
                            } else {
                                None
                            };

                            (is_dir, is_symlink, size, mode, symlink_target)
                        } else {
                            (path.is_dir(), false, 0, 0, None)
                        };

                    entries.push(FileEntry {
                        name,
                        path,
                        is_dir,
                        is_symlink,
                        size,
                        mode,
                        symlink_target,
                    });
                }
            }

            // Sort directories first, then files, then alphabetically
            entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));

            Box::new(ReadDirMsg { id, entries }) as Msg
        })
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

/// Determines whether a file is hidden based on its name.
///
/// This function matches the Go implementation's `IsHidden` function and provides
/// basic hidden file detection for compatibility. It only checks if the filename
/// starts with a dot (dotfile convention on Unix systems).
///
/// For more comprehensive hidden file detection that includes Windows file attributes,
/// use the internal `is_hidden()` function instead.
///
/// # Arguments
///
/// * `name` - The filename to check
///
/// # Returns
///
/// A tuple containing:
/// - `bool`: Whether the file is hidden (starts with '.')
/// - `Option<String>`: Always `None` in this implementation (for Go compatibility)
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::filepicker::is_hidden_name;
///
/// let (hidden, _) = is_hidden_name(".hidden_file");
/// assert!(hidden);
///
/// let (hidden, _) = is_hidden_name("visible_file.txt");
/// assert!(!hidden);
/// ```
pub fn is_hidden_name(name: &str) -> (bool, Option<String>) {
    let is_hidden = name.starts_with('.');
    (is_hidden, None)
}

/// Determines whether a file or directory should be considered hidden.
///
/// This function implements cross-platform hidden file detection:
/// - On Windows: Checks the FILE_ATTRIBUTE_HIDDEN attribute, with dotfiles as fallback
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
    is_hidden_impl(path, name)
}

fn is_hidden_impl(path: &Path, name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        // On Windows, check file attributes for hidden flag
        if let Ok(metadata) = std::fs::metadata(path) {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;

            // Check if file has hidden attribute
            if metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 {
                return true;
            }
        }

        // Fallback: also consider dotfiles as hidden on Windows
        name.starts_with('.')
    }
    #[cfg(not(target_os = "windows"))]
    {
        // On Unix-like systems, files starting with '.' are hidden
        let _ = path; // Unused on Unix systems
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
        // Handle window size messages FIRST to ensure height is set correctly
        if let Some(_window_msg) = msg.downcast_ref::<bubbletea_rs::WindowSizeMsg>() {
            if self.auto_height {
                self.height = (_window_msg.height as usize).saturating_sub(MARGIN_BOTTOM);
            }
            // Update max based on new height, but ensure it doesn't exceed file count
            self.max = if self.files.is_empty() {
                self.height.saturating_sub(1)
            } else {
                std::cmp::min(
                    self.height.saturating_sub(1),
                    self.files.len().saturating_sub(1),
                )
            };

            // Adjust min if necessary to keep viewport consistent
            if self.max < self.selected {
                self.min = self.selected.saturating_sub(self.height.saturating_sub(1));
                self.max = self.selected;
            }
            return None;
        }

        // Handle readDirMsg and errorMsg (would be async in real implementation)
        if let Some(read_dir_msg) = msg.downcast_ref::<ReadDirMsg>() {
            if read_dir_msg.id == self.id {
                self.files = read_dir_msg.entries.clone();

                // Calculate max properly based on current height and file count
                if self.files.is_empty() {
                    self.max = 0;
                } else {
                    // Ensure max doesn't exceed available files or viewport height
                    let viewport_max = self.height.saturating_sub(1);
                    let file_max = self.files.len().saturating_sub(1);
                    self.max = std::cmp::min(viewport_max, file_max);

                    // Ensure selected index is within bounds
                    if self.selected >= self.files.len() {
                        self.selected = file_max;
                    }

                    // Adjust viewport if selected item is outside current view
                    if self.selected > self.max {
                        self.min = self.selected.saturating_sub(viewport_max);
                        self.max = self.selected;
                    } else if self.selected < self.min {
                        self.min = self.selected;
                        self.max = std::cmp::min(self.min + viewport_max, file_max);
                    }
                }
            }
            return None;
        }

        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            match key_msg {
                key_msg if self.keymap.go_to_top.matches(key_msg) => {
                    self.selected = 0;
                    self.min = 0;
                    self.max = self.height.saturating_sub(1);
                }
                key_msg if self.keymap.go_to_last.matches(key_msg) => {
                    self.selected = self.files.len().saturating_sub(1);
                    self.min = self.files.len().saturating_sub(self.height);
                    self.max = self.files.len().saturating_sub(1);
                }
                key_msg if self.keymap.down.matches(key_msg) => {
                    if self.selected < self.files.len().saturating_sub(1) {
                        self.selected += 1;
                    }
                    if self.selected > self.max {
                        self.min += 1;
                        self.max += 1;
                    }
                }
                key_msg if self.keymap.up.matches(key_msg) => {
                    self.selected = self.selected.saturating_sub(1);
                    if self.selected < self.min {
                        self.min = self.min.saturating_sub(1);
                        self.max = self.max.saturating_sub(1);
                    }
                }
                key_msg if self.keymap.page_down.matches(key_msg) => {
                    self.selected += self.height;
                    if self.selected >= self.files.len() {
                        self.selected = self.files.len().saturating_sub(1);
                    }
                    self.min += self.height;
                    self.max += self.height;

                    if self.max >= self.files.len() {
                        self.max = self.files.len().saturating_sub(1);
                        self.min = self.max.saturating_sub(self.height);
                    }
                }
                key_msg if self.keymap.page_up.matches(key_msg) => {
                    self.selected = self.selected.saturating_sub(self.height);
                    self.min = self.min.saturating_sub(self.height);
                    self.max = self.max.saturating_sub(self.height);

                    if self.min == 0 {
                        self.min = 0;
                        self.max = self.min + self.height;
                    }
                }
                key_msg if self.keymap.back.matches(key_msg) => {
                    if let Some(parent) = self.current_directory.parent() {
                        self.current_directory = parent.to_path_buf();
                        if !self.selected_stack.is_empty() {
                            let (selected, min, max) = self.pop_view();
                            self.selected = selected;
                            self.min = min;
                            self.max = max;
                        } else {
                            self.selected = 0;
                            self.min = 0;
                            self.max = self.height.saturating_sub(1);
                        }
                        self.read_dir();
                    }
                }
                key_msg if self.keymap.open.matches(key_msg) && !self.files.is_empty() => {
                    let f = &self.files[self.selected].clone();
                    let mut is_dir = f.is_dir;

                    // Handle symlinks
                    if f.is_symlink {
                        if let Some(target) = &f.symlink_target {
                            if target.is_dir() {
                                is_dir = true;
                            }
                        }
                    }

                    // Check if we can select this file/directory
                    if ((!is_dir && self.file_allowed) || (is_dir && self.dir_allowed))
                        && self.keymap.select.matches(key_msg)
                    {
                        // Select the current path as the selection
                        self.path = f.path.to_string_lossy().to_string();
                    }

                    // Navigate into directory
                    if is_dir {
                        self.push_view(self.selected, self.min, self.max);
                        self.current_directory = f.path.clone();
                        self.selected = 0;
                        self.min = 0;
                        self.max = self.height.saturating_sub(1);
                        self.read_dir();
                    } else {
                        // Set the selected file path
                        self.path = f.path.to_string_lossy().to_string();
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn view(&self) -> String {
        // Display error if present
        if let Some(error) = &self.error {
            return self
                .styles
                .empty_directory
                .clone()
                .height(self.height as i32)
                .max_height(self.height as i32)
                .render(error);
        }

        if self.files.is_empty() {
            return self
                .styles
                .empty_directory
                .clone()
                .height(self.height as i32)
                .max_height(self.height as i32)
                .render("Bummer. No Files Found.");
        }

        let mut output = String::new();

        for (i, f) in self.files.iter().enumerate() {
            if i < self.min || i > self.max {
                continue;
            }

            let size = format_file_size(f.size);
            let disabled = !self.can_select(&f.name) && !f.is_dir;

            if self.selected == i {
                let mut selected_line = String::new();

                if self.show_permissions {
                    selected_line.push(' ');
                    selected_line.push_str(&format_mode(f.mode));
                }

                if self.show_size {
                    selected_line.push_str(&format!("{:>width$}", size, width = FILE_SIZE_WIDTH));
                }

                selected_line.push(' ');
                selected_line.push_str(&f.name);

                if f.is_symlink {
                    if let Some(target) = &f.symlink_target {
                        selected_line.push_str(" → ");
                        selected_line.push_str(&target.to_string_lossy());
                    }
                }

                if disabled {
                    output.push_str(&self.styles.disabled_cursor.render(&self.cursor));
                    output.push_str(&self.styles.disabled_selected.render(&selected_line));
                } else {
                    output.push_str(&self.styles.cursor.render(&self.cursor));
                    output.push_str(&self.styles.selected.render(&selected_line));
                }
                output.push('\n');
                continue;
            }

            // Non-selected items
            let style = if f.is_dir {
                &self.styles.directory
            } else if f.is_symlink {
                &self.styles.symlink
            } else if disabled {
                &self.styles.disabled_file
            } else {
                &self.styles.file
            };

            let mut file_name = style.render(&f.name);
            output.push_str(&self.styles.cursor.render(" "));

            if f.is_symlink {
                if let Some(target) = &f.symlink_target {
                    file_name.push_str(" → ");
                    file_name.push_str(&target.to_string_lossy());
                }
            }

            if self.show_permissions {
                output.push(' ');
                output.push_str(&self.styles.permission.render(&format_mode(f.mode)));
            }

            if self.show_size {
                output.push_str(&self.styles.file_size.render(&size));
            }

            output.push(' ');
            output.push_str(&file_name);
            output.push('\n');
        }

        // Pad to fill height
        let current_height = output.lines().count();
        for _ in current_height..=self.height {
            output.push('\n');
        }

        output
    }
}

/// Formats file size in human-readable format, similar to go-humanize.
///
/// Converts byte sizes into readable format using decimal units (1000-based).
/// This matches the behavior of the Go humanize library used in the original implementation.
///
/// # Arguments
///
/// * `size` - The file size in bytes
///
/// # Returns
///
/// A formatted string representation of the file size (e.g., "1.2kB", "45MB")
///
/// # Examples
///
/// ```rust
/// // Note: This is a private function, shown for documentation purposes
/// // format_file_size(1024) -> "1.0kB"
/// // format_file_size(1500000) -> "1.5MB"
/// ```
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "kB", "MB", "GB", "TB", "PB"];

    if size == 0 {
        return "0B".to_string();
    }

    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1000.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1000.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{}B", size)
    } else if size_f >= 100.0 {
        format!("{:.0}{}", size_f, UNITS[unit_index])
    } else {
        format!("{:.1}{}", size_f, UNITS[unit_index])
    }
}

/// Formats file mode/permissions in Unix style.
///
/// Converts Unix file permission bits into a human-readable string representation
/// similar to what `ls -l` displays. The format is a 10-character string where:
/// - First character: file type (d=directory, l=symlink, -=regular file, etc.)
/// - Next 9 characters: permissions in rwx format for owner, group, and others
///
/// # Arguments
///
/// * `mode` - The file mode bits from file metadata
///
/// # Returns
///
/// A 10-character permission string (e.g., "drwxr-xr-x", "-rw-r--r--")
///
/// # Examples
///
/// ```rust
/// // Note: This is a private function, shown for documentation purposes
/// // format_mode(0o755) -> "-rwxr-xr-x"
/// // format_mode(0o644) -> "-rw-r--r--"
/// ```
#[cfg(unix)]
fn format_mode(mode: u32) -> String {
    // Use standard Rust constants instead of libc for better type compatibility
    const S_IFMT: u32 = 0o170000;
    const S_IFDIR: u32 = 0o040000;
    const S_IFLNK: u32 = 0o120000;
    const S_IFBLK: u32 = 0o060000;
    const S_IFCHR: u32 = 0o020000;
    const S_IFIFO: u32 = 0o010000;
    const S_IFSOCK: u32 = 0o140000;

    const S_IRUSR: u32 = 0o400;
    const S_IWUSR: u32 = 0o200;
    const S_IXUSR: u32 = 0o100;
    const S_IRGRP: u32 = 0o040;
    const S_IWGRP: u32 = 0o020;
    const S_IXGRP: u32 = 0o010;
    const S_IROTH: u32 = 0o004;
    const S_IWOTH: u32 = 0o002;
    const S_IXOTH: u32 = 0o001;

    let file_type = match mode & S_IFMT {
        S_IFDIR => 'd',
        S_IFLNK => 'l',
        S_IFBLK => 'b',
        S_IFCHR => 'c',
        S_IFIFO => 'p',
        S_IFSOCK => 's',
        _ => '-',
    };

    let owner_perms = format!(
        "{}{}{}",
        if mode & S_IRUSR != 0 { 'r' } else { '-' },
        if mode & S_IWUSR != 0 { 'w' } else { '-' },
        if mode & S_IXUSR != 0 { 'x' } else { '-' }
    );

    let group_perms = format!(
        "{}{}{}",
        if mode & S_IRGRP != 0 { 'r' } else { '-' },
        if mode & S_IWGRP != 0 { 'w' } else { '-' },
        if mode & S_IXGRP != 0 { 'x' } else { '-' }
    );

    let other_perms = format!(
        "{}{}{}",
        if mode & S_IROTH != 0 { 'r' } else { '-' },
        if mode & S_IWOTH != 0 { 'w' } else { '-' },
        if mode & S_IXOTH != 0 { 'x' } else { '-' }
    );

    format!("{}{}{}{}", file_type, owner_perms, group_perms, other_perms)
}

/// Formats file mode/permissions on non-Unix systems.
///
/// On non-Unix systems (primarily Windows), file permissions don't use
/// the Unix rwx model, so this function returns a placeholder string.
///
/// # Arguments
///
/// * `_mode` - The file mode (ignored on non-Unix systems)
///
/// # Returns
///
/// A placeholder permission string "----------"
#[cfg(not(unix))]
fn format_mode(_mode: u32) -> String {
    "----------".to_string()
}
