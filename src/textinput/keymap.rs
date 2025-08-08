//! Key bindings for the textinput component.

use crate::key::{new_binding, with_keys_str, Binding};

/// KeyMap is the key bindings for different actions within the textinput.
/// Matches Go's KeyMap struct exactly.
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Move cursor one character right.
    pub character_forward: Binding,
    /// Move cursor one character left.
    pub character_backward: Binding,
    /// Move cursor one word right.
    pub word_forward: Binding,
    /// Move cursor one word left.
    pub word_backward: Binding,
    /// Delete the previous word.
    pub delete_word_backward: Binding,
    /// Delete the next word.
    pub delete_word_forward: Binding,
    /// Delete from cursor to end of line.
    pub delete_after_cursor: Binding,
    /// Delete from start of line to cursor.
    pub delete_before_cursor: Binding,
    /// Delete one character backward.
    pub delete_character_backward: Binding,
    /// Delete one character forward.
    pub delete_character_forward: Binding,
    /// Move to start of line.
    pub line_start: Binding,
    /// Move to end of line.
    pub line_end: Binding,
    /// Paste from clipboard.
    pub paste: Binding,
    /// Accept the current suggestion.
    pub accept_suggestion: Binding,
    /// Move to the next suggestion.
    pub next_suggestion: Binding,
    /// Move to the previous suggestion.
    pub prev_suggestion: Binding,
}

/// DefaultKeyMap is the default set of key bindings for navigating and acting
/// upon the textinput. Matches Go's DefaultKeyMap exactly.
pub fn default_key_map() -> KeyMap {
    KeyMap {
        character_forward: new_binding(vec![with_keys_str(&["right", "ctrl+f"])]),
        character_backward: new_binding(vec![with_keys_str(&["left", "ctrl+b"])]),
        word_forward: new_binding(vec![with_keys_str(&["alt+right", "ctrl+right", "alt+f"])]),
        word_backward: new_binding(vec![with_keys_str(&["alt+left", "ctrl+left", "alt+b"])]),
        delete_word_backward: new_binding(vec![with_keys_str(&["alt+backspace", "ctrl+w"])]),
        delete_word_forward: new_binding(vec![with_keys_str(&["alt+delete", "alt+d"])]),
        delete_after_cursor: new_binding(vec![with_keys_str(&["ctrl+k"])]),
        delete_before_cursor: new_binding(vec![with_keys_str(&["ctrl+u"])]),
        delete_character_backward: new_binding(vec![with_keys_str(&["backspace", "ctrl+h"])]),
        delete_character_forward: new_binding(vec![with_keys_str(&["delete", "ctrl+d"])]),
        line_start: new_binding(vec![with_keys_str(&["home", "ctrl+a"])]),
        line_end: new_binding(vec![with_keys_str(&["end", "ctrl+e"])]),
        paste: new_binding(vec![with_keys_str(&["ctrl+v"])]),
        accept_suggestion: new_binding(vec![with_keys_str(&["tab"])]),
        next_suggestion: new_binding(vec![with_keys_str(&["down", "ctrl+n"])]),
        prev_suggestion: new_binding(vec![with_keys_str(&["up", "ctrl+p"])]),
    }
}
