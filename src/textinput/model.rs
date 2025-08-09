//! Core model implementation for the textinput component.

use super::keymap::{default_key_map, KeyMap};
#[cfg(feature = "clipboard-support")]
use super::types::PasteMsg;
use super::types::{EchoMode, PasteErrMsg, ValidateFunc};
use crate::cursor::{new as cursor_new, Model as Cursor};
use bubbletea_rs::{Cmd, Model as BubbleTeaModel, Msg};
use lipgloss_extras::prelude::*;
use std::time::Duration;

/// The main text input component model for Bubble Tea applications.
///
/// This struct represents a single-line text input field with support for:
/// - Cursor movement and text editing
/// - Input validation
/// - Auto-completion suggestions  
/// - Different echo modes (normal, password, hidden)
/// - Horizontal scrolling for long text
/// - Customizable styling and key bindings
///
/// The model follows the Elm Architecture pattern used by Bubble Tea, with
/// separate `Init()`, `Update()`, and `View()` methods for state management.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::textinput::{new, EchoMode};
/// use bubbletea_rs::Model;
///
/// // Create and configure a text input
/// let mut input = new();
/// input.focus();
/// input.set_placeholder("Enter your name...");
/// input.set_width(30);
/// input.set_char_limit(50);
///
/// // For password input
/// input.set_echo_mode(EchoMode::EchoPassword);
///
/// // With validation
/// input.set_validate(Box::new(|s: &str| {
///     if s.len() >= 3 {
///         Ok(())
///     } else {
///         Err("Must be at least 3 characters".to_string())
///     }
/// }));
/// ```
///
/// # Note
///
/// This struct matches the Go `Model` struct exactly for 1-1 compatibility.
#[allow(dead_code)]
pub struct Model {
    /// Err is an error that was not caught by a validator.
    pub err: Option<String>,

    /// Prompt is the prompt to display before the text input.
    pub prompt: String,
    /// Style for the prompt prefix.
    pub prompt_style: Style,

    /// TextStyle is the style of the text as it's being typed.
    pub text_style: Style,

    /// Placeholder is the placeholder text to display when the input is empty.
    pub placeholder: String,
    /// Style for the placeholder text.
    pub placeholder_style: Style,

    /// Cursor is the cursor model.
    pub cursor: Cursor,
    /// Cursor rendering mode (blink/static/hidden).
    pub cursor_mode: crate::cursor::Mode,

    /// Value is the value of the text input.
    pub(super) value: Vec<char>,

    /// Focus indicates whether the input is focused.
    pub(super) focus: bool,

    /// Position is the cursor position.
    pub(super) pos: usize,

    /// Width is the maximum number of characters that can be displayed at once.
    pub width: i32,

    /// KeyMap encodes the keybindings.
    pub key_map: KeyMap,

    /// CharLimit is the maximum number of characters this input will accept.
    /// 0 means no limit.
    pub char_limit: i32,

    /// EchoMode is the echo mode of the input.
    pub echo_mode: EchoMode,

    /// EchoCharacter is the character to use for password fields.
    pub echo_character: char,

    /// CompletionStyle is the style of the completion suggestion.
    pub completion_style: Style,

    /// Validate is a function that validates the input.
    pub(super) validate: Option<ValidateFunc>,

    /// Internal fields for managing overflow and suggestions
    pub(super) offset: usize,
    pub(super) offset_right: usize,
    pub(super) suggestions: Vec<Vec<char>>,
    pub(super) matched_suggestions: Vec<Vec<char>>,
    pub(super) show_suggestions: bool,
    pub(super) current_suggestion_index: usize,
}

/// Creates a new text input model with default settings.
///
/// The returned model is not focused by default. Call `focus()` to enable keyboard input.
///
/// # Returns
///
/// A new `Model` instance with default configuration:
/// - Empty value and placeholder
/// - Default prompt ("> ")
/// - Normal echo mode
/// - No character or width limits
/// - Default key bindings
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::textinput::new;
///
/// let mut input = new();
/// input.focus();
/// input.set_placeholder("Enter text...");
/// input.set_width(30);
/// ```
///
/// # Note
///
/// This function matches Go's New function exactly for compatibility.
pub fn new() -> Model {
    let mut m = Model {
        err: None,
        prompt: "> ".to_string(),
        prompt_style: Style::new(),
        text_style: Style::new(),
        placeholder: String::new(),
        placeholder_style: Style::new().foreground(Color::from("240")),
        cursor: cursor_new(),
        cursor_mode: crate::cursor::Mode::Blink,
        value: Vec::new(),
        focus: false,
        pos: 0,
        width: 0,
        key_map: default_key_map(),
        char_limit: 0,
        echo_mode: EchoMode::EchoNormal,
        echo_character: '*',
        completion_style: Style::new().foreground(Color::from("240")),
        validate: None,
        offset: 0,
        offset_right: 0,
        suggestions: Vec::new(),
        matched_suggestions: Vec::new(),
        show_suggestions: false,
        current_suggestion_index: 0,
    };

    m.cursor.set_mode(crate::cursor::Mode::Blink);
    m
}

/// Creates a new text input model (alias for `new()`).
///
/// This is provided for compatibility with the bubbletea pattern where both
/// `New()` and `NewModel()` functions exist.
///
/// # Returns
///
/// A new `Model` instance with default settings
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::textinput::new_model;
///
/// let input = new_model();
/// ```
///
/// # Note
///
/// The Go implementation has both New() and NewModel() functions for compatibility.
pub fn new_model() -> Model {
    new()
}

impl Default for Model {
    fn default() -> Self {
        new()
    }
}

/// Creates a command that triggers cursor blinking.
///
/// This command should be returned from your application's `init()` method or
/// when focusing the text input to start the cursor blinking animation.
///
/// # Returns
///
/// A `Cmd` that will periodically send blink messages to animate the cursor
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::textinput::blink;
/// use bubbletea_rs::{Model, Cmd};
///
/// struct App {
///     // ... other fields
/// }
///
/// impl Model for App {
///     fn init() -> (Self, Option<Cmd>) {
///         // Return blink command to start cursor animation
///         (App { /* ... */ }, Some(blink()))
///     }
/// #
/// #   fn update(&mut self, _msg: bubbletea_rs::Msg) -> Option<Cmd> { None }
/// #   fn view(&self) -> String { String::new() }
/// }
/// ```
pub fn blink() -> Cmd {
    use bubbletea_rs::tick as bubbletea_tick;
    let id = 0usize;
    let tag = 0usize;
    bubbletea_tick(Duration::from_millis(500), move |_| {
        Box::new(crate::cursor::BlinkMsg { id, tag }) as Msg
    })
}

/// Creates a command that retrieves text from the system clipboard.
///
/// This command reads the current clipboard contents and sends a paste message
/// that can be handled by the text input's `update()` method.
///
/// # Returns
///
/// A `Cmd` that will attempt to read from clipboard and send either:
/// - `PasteMsg(String)` with the clipboard contents on success
/// - `PasteErrMsg(String)` with an error message on failure
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::textinput::paste;
///
/// // This is typically called internally when Ctrl+V is pressed
/// // but can be used manually:
/// let paste_cmd = paste();
/// ```
///
/// # Errors
///
/// The returned command may produce a `PasteErrMsg` if:
/// - The clipboard is not accessible
/// - The clipboard contains non-text data
/// - System clipboard permissions are denied
pub fn paste() -> Cmd {
    use bubbletea_rs::tick as bubbletea_tick;
    bubbletea_tick(Duration::from_nanos(1), |_| {
        #[cfg(feature = "clipboard-support")]
        {
            use clipboard::{ClipboardContext, ClipboardProvider};
            let res: Result<String, String> = (|| {
                let mut ctx: ClipboardContext = ClipboardProvider::new()
                    .map_err(|e| format!("Failed to create clipboard context: {}", e))?;
                ctx.get_contents()
                    .map_err(|e| format!("Failed to read clipboard: {}", e))
            })();
            match res {
                Ok(s) => Box::new(PasteMsg(s)) as Msg,
                Err(e) => Box::new(PasteErrMsg(e)) as Msg,
            }
        }
        #[cfg(not(feature = "clipboard-support"))]
        {
            Box::new(PasteErrMsg("Clipboard support not enabled".to_string())) as Msg
        }
    })
}

impl BubbleTeaModel for Model {
    fn init() -> (Self, std::option::Option<Cmd>) {
        let model = new();
        (model, std::option::Option::None)
    }

    fn update(&mut self, msg: Msg) -> std::option::Option<Cmd> {
        self.update(msg)
    }

    fn view(&self) -> String {
        self.view()
    }
}
