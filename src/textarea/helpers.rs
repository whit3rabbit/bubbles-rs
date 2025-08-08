//! Helper functions and types for the textarea component.
//!
//! This module exposes:
//! - **`TextareaKeyMap`**: default key bindings (movement, deletion, clipboard, advanced word ops).
//! - **`TextareaStyle`**: theming knobs for base/text, prompt, line numbers, cursor line, etc.
//! - Utility helpers used by `textarea::Model` (word boundaries, clamping, spacing).
//!
//! ### Styling
//! Use `default_focused_style()` and `default_blurred_style()` for presets that
//! mimic the upstream Go defaults, or construct a custom `TextareaStyle` and
//! assign it to the model.
//!
//! ```rust
//! use bubbles_rs::textarea::{helpers::TextareaStyle, new};
//! use lipgloss::Style;
//!
//! let mut model = new();
//! let custom = TextareaStyle {
//!     base: Style::new(),
//!     text: Style::new(),
//!     prompt: Style::new().foreground("#04b575"),
//!     line_number: Style::new().foreground("#666666"),
//!     cursor_line: Style::new().background("#2a2a2a"),
//!     cursor_line_number: Style::new().foreground("#666666"),
//!     end_of_buffer: Style::new().foreground("#3c3c3c"),
//!     placeholder: Style::new().foreground("#666666"),
//! };
//! model.focused_style = custom.clone();
//! model.blurred_style = custom;
//! ```

use crate::key::{self, KeyPress};
use crossterm::event::{KeyCode, KeyModifiers};
use lipgloss::Style;

/// Complete KeyMap for textarea component - direct port from Go
#[derive(Debug, Clone)]
pub struct TextareaKeyMap {
    /// Move cursor one character left.
    pub character_backward: key::Binding,
    /// Move cursor one character right.
    pub character_forward: key::Binding,
    /// Delete from cursor to end of line.
    pub delete_after_cursor: key::Binding,
    /// Delete from start of line to cursor.
    pub delete_before_cursor: key::Binding,
    /// Delete one character backward.
    pub delete_character_backward: key::Binding,
    /// Delete one character forward.
    pub delete_character_forward: key::Binding,
    /// Delete previous word.
    pub delete_word_backward: key::Binding,
    /// Delete next word.
    pub delete_word_forward: key::Binding,
    /// Insert newline.
    pub insert_newline: key::Binding,
    /// Move cursor to end of line.
    pub line_end: key::Binding,
    /// Move cursor to next visual line.
    pub line_next: key::Binding,
    /// Move cursor to previous visual line.
    pub line_previous: key::Binding,
    /// Move cursor to start of line.
    pub line_start: key::Binding,
    /// Paste from clipboard.
    pub paste: key::Binding,
    /// Move one word left.
    pub word_backward: key::Binding,
    /// Move one word right.
    pub word_forward: key::Binding,
    /// Move to beginning of input.
    pub input_begin: key::Binding,
    /// Move to end of input.
    pub input_end: key::Binding,
    // Advanced bindings from Go
    /// Uppercase the word to the right of the cursor.
    pub uppercase_word_forward: key::Binding,
    /// Lowercase the word to the right of the cursor.
    pub lowercase_word_forward: key::Binding,
    /// Capitalize the word to the right of the cursor.
    pub capitalize_word_forward: key::Binding,
    /// Transpose the character to the left with the current one.
    pub transpose_character_backward: key::Binding,
}

/// Implementation of KeyMap trait for help integration
impl crate::key::KeyMap for TextareaKeyMap {
    fn short_help(&self) -> Vec<&key::Binding> {
        vec![
            &self.character_backward,
            &self.character_forward,
            &self.line_next,
            &self.line_previous,
            &self.insert_newline,
            &self.delete_character_backward,
        ]
    }

    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![
            vec![
                &self.character_backward,
                &self.character_forward,
                &self.word_backward,
                &self.word_forward,
            ],
            vec![
                &self.line_next,
                &self.line_previous,
                &self.line_start,
                &self.line_end,
            ],
            vec![
                &self.insert_newline,
                &self.delete_character_backward,
                &self.delete_character_forward,
                &self.paste,
            ],
            vec![
                &self.delete_word_backward,
                &self.delete_word_forward,
                &self.delete_after_cursor,
                &self.delete_before_cursor,
            ],
        ]
    }
}

impl Default for TextareaKeyMap {
    fn default() -> Self {
        Self {
            character_forward: key::Binding::new(vec![
                KeyPress::from(KeyCode::Right),
                KeyPress::from((KeyCode::Char('f'), KeyModifiers::CONTROL)),
            ])
            .with_help("→/ctrl+f", "character forward"),

            character_backward: key::Binding::new(vec![
                KeyPress::from(KeyCode::Left),
                KeyPress::from((KeyCode::Char('b'), KeyModifiers::CONTROL)),
            ])
            .with_help("←/ctrl+b", "character backward"),

            word_forward: key::Binding::new(vec![
                KeyPress::from((KeyCode::Right, KeyModifiers::ALT)),
                KeyPress::from((KeyCode::Char('f'), KeyModifiers::ALT)),
            ])
            .with_help("alt+→/alt+f", "word forward"),

            word_backward: key::Binding::new(vec![
                KeyPress::from((KeyCode::Left, KeyModifiers::ALT)),
                KeyPress::from((KeyCode::Char('b'), KeyModifiers::ALT)),
            ])
            .with_help("alt+←/alt+b", "word backward"),

            line_next: key::Binding::new(vec![
                KeyPress::from(KeyCode::Down),
                KeyPress::from((KeyCode::Char('n'), KeyModifiers::CONTROL)),
            ])
            .with_help("↓/ctrl+n", "next line"),

            line_previous: key::Binding::new(vec![
                KeyPress::from(KeyCode::Up),
                KeyPress::from((KeyCode::Char('p'), KeyModifiers::CONTROL)),
            ])
            .with_help("↑/ctrl+p", "previous line"),

            delete_word_backward: key::Binding::new(vec![
                KeyPress::from((KeyCode::Backspace, KeyModifiers::ALT)),
                KeyPress::from((KeyCode::Char('w'), KeyModifiers::CONTROL)),
            ])
            .with_help("alt+backspace/ctrl+w", "delete word backward"),

            delete_word_forward: key::Binding::new(vec![
                KeyPress::from((KeyCode::Delete, KeyModifiers::ALT)),
                KeyPress::from((KeyCode::Char('d'), KeyModifiers::ALT)),
            ])
            .with_help("alt+delete/alt+d", "delete word forward"),

            delete_after_cursor: key::Binding::new(vec![KeyPress::from((
                KeyCode::Char('k'),
                KeyModifiers::CONTROL,
            ))])
            .with_help("ctrl+k", "delete after cursor"),

            delete_before_cursor: key::Binding::new(vec![KeyPress::from((
                KeyCode::Char('u'),
                KeyModifiers::CONTROL,
            ))])
            .with_help("ctrl+u", "delete before cursor"),

            insert_newline: key::Binding::new(vec![
                KeyPress::from(KeyCode::Enter),
                KeyPress::from((KeyCode::Char('m'), KeyModifiers::CONTROL)),
            ])
            .with_help("enter/ctrl+m", "insert newline"),

            delete_character_backward: key::Binding::new(vec![
                KeyPress::from(KeyCode::Backspace),
                KeyPress::from((KeyCode::Char('h'), KeyModifiers::CONTROL)),
            ])
            .with_help("backspace/ctrl+h", "delete character backward"),

            delete_character_forward: key::Binding::new(vec![
                KeyPress::from(KeyCode::Delete),
                KeyPress::from((KeyCode::Char('d'), KeyModifiers::CONTROL)),
            ])
            .with_help("delete/ctrl+d", "delete character forward"),

            line_start: key::Binding::new(vec![
                KeyPress::from(KeyCode::Home),
                KeyPress::from((KeyCode::Char('a'), KeyModifiers::CONTROL)),
            ])
            .with_help("home/ctrl+a", "line start"),

            line_end: key::Binding::new(vec![
                KeyPress::from(KeyCode::End),
                KeyPress::from((KeyCode::Char('e'), KeyModifiers::CONTROL)),
            ])
            .with_help("end/ctrl+e", "line end"),

            paste: key::Binding::new(vec![KeyPress::from((
                KeyCode::Char('v'),
                KeyModifiers::CONTROL,
            ))])
            .with_help("ctrl+v", "paste"),

            input_begin: key::Binding::new(vec![
                KeyPress::from((KeyCode::Char('<'), KeyModifiers::ALT)),
                KeyPress::from((KeyCode::Home, KeyModifiers::CONTROL)),
            ])
            .with_help("alt+</ctrl+home", "input begin"),

            input_end: key::Binding::new(vec![
                KeyPress::from((KeyCode::Char('>'), KeyModifiers::ALT)),
                KeyPress::from((KeyCode::End, KeyModifiers::CONTROL)),
            ])
            .with_help("alt+>/ctrl+end", "input end"),

            capitalize_word_forward: key::Binding::new(vec![KeyPress::from((
                KeyCode::Char('c'),
                KeyModifiers::ALT,
            ))])
            .with_help("alt+c", "capitalize word forward"),

            lowercase_word_forward: key::Binding::new(vec![KeyPress::from((
                KeyCode::Char('l'),
                KeyModifiers::ALT,
            ))])
            .with_help("alt+l", "lowercase word forward"),

            uppercase_word_forward: key::Binding::new(vec![KeyPress::from((
                KeyCode::Char('u'),
                KeyModifiers::ALT,
            ))])
            .with_help("alt+u", "uppercase word forward"),

            transpose_character_backward: key::Binding::new(vec![KeyPress::from((
                KeyCode::Char('t'),
                KeyModifiers::CONTROL,
            ))])
            .with_help("ctrl+t", "transpose character backward"),
        }
    }
}

/// Style that will be applied to the text area - direct port from Go
#[derive(Debug, Clone)]
pub struct TextareaStyle {
    /// Base style applied to the entire textarea view.
    pub base: Style,
    /// Style for the current cursor line background.
    pub cursor_line: Style,
    /// Style for the current line number.
    pub cursor_line_number: Style,
    /// Style for the end-of-buffer character.
    pub end_of_buffer: Style,
    /// Style for line numbers generally.
    pub line_number: Style,
    /// Style for placeholder text.
    pub placeholder: Style,
    /// Style for the prompt prefix.
    pub prompt: Style,
    /// Style for regular text content.
    pub text: Style,
}

impl TextareaStyle {
    /// Computed cursor line style
    pub fn computed_cursor_line(&self) -> Style {
        self.cursor_line
            .clone()
            .inherit(self.base.clone())
            .inline(true)
    }

    /// Computed cursor line number style  
    pub fn computed_cursor_line_number(&self) -> Style {
        self.cursor_line_number
            .clone()
            .inherit(self.cursor_line.clone())
            .inherit(self.base.clone())
            .inline(true)
    }

    /// Computed end of buffer style
    pub fn computed_end_of_buffer(&self) -> Style {
        self.end_of_buffer
            .clone()
            .inherit(self.base.clone())
            .inline(true)
    }

    /// Computed line number style
    pub fn computed_line_number(&self) -> Style {
        self.line_number
            .clone()
            .inherit(self.base.clone())
            .inline(true)
    }

    /// Computed placeholder style
    pub fn computed_placeholder(&self) -> Style {
        self.placeholder
            .clone()
            .inherit(self.base.clone())
            .inline(true)
    }

    /// Computed prompt style
    pub fn computed_prompt(&self) -> Style {
        self.prompt.clone().inherit(self.base.clone()).inline(true)
    }

    /// Computed text style
    pub fn computed_text(&self) -> Style {
        self.text.clone().inherit(self.base.clone()).inline(true)
    }
}

/// Create default focused style - matching Go DefaultStyles
pub fn default_focused_style() -> TextareaStyle {
    TextareaStyle {
        base: Style::new(),
        cursor_line: Style::new().background("#2a2a2a"),
        cursor_line_number: Style::new().foreground("#666666"),
        end_of_buffer: Style::new().foreground("#3c3c3c"),
        line_number: Style::new().foreground("#666666"),
        placeholder: Style::new().foreground("#666666"),
        prompt: Style::new().foreground("#04b575"),
        text: Style::new(),
    }
}

/// Create default blurred style - matching Go DefaultStyles
pub fn default_blurred_style() -> TextareaStyle {
    TextareaStyle {
        base: Style::new(),
        cursor_line: Style::new(),
        cursor_line_number: Style::new().foreground("#3c3c3c"),
        end_of_buffer: Style::new().foreground("#3c3c3c"),
        line_number: Style::new().foreground("#3c3c3c"),
        placeholder: Style::new().foreground("#666666"),
        prompt: Style::new().foreground("#666666"),
        text: Style::new(),
    }
}

/// Create default key map for textarea - function version
pub fn default_key_map() -> TextareaKeyMap {
    TextareaKeyMap::default()
}

/// Check if a character is a word boundary
pub fn is_word_boundary(ch: char) -> bool {
    ch.is_whitespace() || ch.is_ascii_punctuation()
}

/// Find the start of the current word
pub fn word_start(text: &str, pos: usize) -> usize {
    if pos == 0 {
        return 0;
    }

    let chars: Vec<char> = text.chars().collect();
    let mut i = pos.saturating_sub(1);

    while i > 0 && !is_word_boundary(chars[i]) {
        i -= 1;
    }

    if i > 0 && is_word_boundary(chars[i]) {
        i + 1
    } else {
        i
    }
}

/// Find the end of the current word
pub fn word_end(text: &str, pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut i = pos;

    while i < chars.len() && !is_word_boundary(chars[i]) {
        i += 1;
    }

    i
}

/// Utility function to clamp a value between bounds
pub fn clamp<T: Ord>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Repeat spaces as characters
pub fn repeat_spaces(n: usize) -> Vec<char> {
    std::iter::repeat_n(' ', n).collect()
}
