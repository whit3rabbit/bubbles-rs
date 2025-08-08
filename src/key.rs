//! Key binding component for managing keybindings, ported from the Go version.
//!
//! This module provides types and functions for defining keybindings that can be
//! used for both input handling and generating help views. It offers a type-safe
//! alternative to string-based key matching.

use bubbletea_rs::KeyMsg;
use crossterm::event::{KeyCode, KeyModifiers};

/// Represents a specific key press, combining a `KeyCode` and `KeyModifiers`.
///
/// This provides a structured, type-safe way to define key combinations that can
/// be used throughout the application for consistent key handling.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::KeyPress;
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// // Create a Ctrl+C key press
/// let ctrl_c = KeyPress {
///     code: KeyCode::Char('c'),
///     mods: KeyModifiers::CONTROL,
/// };
///
/// // Create from tuple
/// let alt_f4: KeyPress = (KeyCode::F(4), KeyModifiers::ALT).into();
///
/// // Create from string
/// let escape: KeyPress = "esc".into();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPress {
    /// The key code representing the physical key pressed.
    pub code: KeyCode,
    /// The modifier keys (Ctrl, Alt, Shift) held during the key press.
    pub mods: KeyModifiers,
}

/// Creates a `KeyPress` from a tuple of `(KeyCode, KeyModifiers)`.
///
/// This provides a convenient way to create key press instances from tuples,
/// making it easy to define key combinations inline.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::KeyPress;
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// let save_key: KeyPress = (KeyCode::Char('s'), KeyModifiers::CONTROL).into();
/// let quit_key: KeyPress = (KeyCode::Char('q'), KeyModifiers::CONTROL).into();
/// ```
impl From<(KeyCode, KeyModifiers)> for KeyPress {
    fn from((code, mods): (KeyCode, KeyModifiers)) -> Self {
        Self { code, mods }
    }
}

/// Creates a `KeyPress` from just a `KeyCode` with no modifiers.
///
/// This is useful for simple keys that don't require modifier combinations,
/// such as arrow keys, function keys, or single character keys.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::KeyPress;
/// use crossterm::event::KeyCode;
///
/// let enter_key: KeyPress = KeyCode::Enter.into();
/// let up_arrow: KeyPress = KeyCode::Up.into();
/// let letter_a: KeyPress = KeyCode::Char('a').into();
/// ```
impl From<KeyCode> for KeyPress {
    fn from(code: KeyCode) -> Self {
        Self {
            code,
            mods: KeyModifiers::NONE,
        }
    }
}

/// Creates a `KeyPress` from a string representation.
///
/// This provides a human-readable way to define key combinations using string
/// names. Supports both simple keys and modifier combinations.
///
/// # Supported Formats
///
/// - Simple keys: "enter", "tab", "esc", "space", "up", "down", etc.
/// - Function keys: "f1", "f2", ..., "f12"
/// - Single characters: "a", "1", "?", "/"
/// - Modifier combinations: "ctrl+c", "alt+f4", "shift+tab"
/// - Complex combinations: "ctrl+alt+a"
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::KeyPress;
///
/// let enter: KeyPress = "enter".into();
/// let ctrl_c: KeyPress = "ctrl+c".into();
/// let alt_f4: KeyPress = "alt+f4".into();
/// let page_up: KeyPress = "pgup".into();
/// ```
///
/// # Panics
///
/// This function does not panic. Unknown key combinations will result in
/// a `KeyPress` with `KeyCode::Null`.
impl From<&str> for KeyPress {
    fn from(s: &str) -> Self {
        parse_key_string(s)
    }
}

/// Help information for displaying keybinding documentation.
///
/// This structure contains the human-readable representation of a key binding
/// that will be shown in help views and documentation.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::Help;
///
/// let help = Help {
///     key: "ctrl+s".to_string(),
///     desc: "Save file".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Help {
    /// The human-readable representation of the key combination (e.g., "ctrl+s", "enter").
    pub key: String,
    /// A brief description of what the key binding does.
    pub desc: String,
}

/// Describes a set of keybindings and their associated help text. A `Binding`
/// represents a single semantic action that can be triggered by one or more
/// physical key presses.
#[derive(Debug, Clone, Default)]
pub struct Binding {
    /// The key press combinations that trigger this binding.
    keys: Vec<KeyPress>,
    /// The help information for displaying in help views.
    help: Help,
    /// Whether the binding is currently enabled and should match key events.
    enabled: bool,
}

/// Option type for configuring a Binding during initialization.
pub type BindingOpt = Box<dyn FnOnce(&mut Binding)>;

impl Binding {
    /// Creates a new keybinding with a set of associated key presses.
    ///
    /// The input can be anything convertible into a `Vec<KeyPress>`, making it
    /// ergonomic to define keys with or without modifiers. The binding is
    /// created in an enabled state with empty help text.
    ///
    /// # Arguments
    ///
    /// * `keys` - A vector of items that can be converted to `KeyPress`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// // Binding with multiple keys, some with modifiers
    /// let save_binding = Binding::new(vec![
    ///     (KeyCode::Char('s'), KeyModifiers::CONTROL), // Ctrl+S
    ///     (KeyCode::F(2), KeyModifiers::NONE), // F2 key
    /// ]);
    ///
    /// // Binding from string representations
    /// let quit_binding = Binding::new(vec!["q", "ctrl+c"]);
    /// ```
    pub fn new<K: Into<KeyPress>>(keys: Vec<K>) -> Self {
        Self {
            keys: keys.into_iter().map(Into::into).collect(),
            help: Help::default(),
            enabled: true,
        }
    }

    /// Creates a new binding using builder options.
    ///
    /// This provides a flexible way to create bindings with various configuration
    /// options applied. Similar to Go's NewBinding function.
    ///
    /// # Arguments
    ///
    /// * `opts` - A vector of builder options to configure the binding
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::{Binding, with_keys_str, with_help};
    ///
    /// let save_binding = Binding::new_binding(vec![
    ///     with_keys_str(&["ctrl+s", "f2"]),
    ///     with_help("ctrl+s", "Save the current file"),
    /// ]);
    /// ```
    pub fn new_binding(opts: Vec<BindingOpt>) -> Self {
        let mut binding = Self {
            keys: Vec::new(),
            help: Help::default(),
            enabled: true,
        };
        for opt in opts {
            opt(&mut binding);
        }
        binding
    }

    /// Sets the help text for the keybinding using a builder pattern.
    ///
    /// # Arguments
    ///
    /// * `key` - The human-readable key representation for help display
    /// * `desc` - A brief description of what the binding does
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let binding = Binding::new(vec![KeyCode::Enter])
    ///     .with_help("enter", "Confirm selection");
    /// ```
    pub fn with_help(mut self, key: impl Into<String>, desc: impl Into<String>) -> Self {
        self.help = Help {
            key: key.into(),
            desc: desc.into(),
        };
        self
    }

    /// Sets the initial enabled state of the keybinding.
    ///
    /// Disabled bindings will not match key events and will not appear in help views.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the binding should be enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let disabled_binding = Binding::new(vec![KeyCode::F(1)])
    ///     .with_enabled(false);
    /// ```
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the keybinding to disabled state (convenience method).
    ///
    /// This is equivalent to calling `with_enabled(false)` but more readable
    /// when you specifically want to disable a binding.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let disabled_binding = Binding::new(vec![KeyCode::F(1)])
    ///     .with_disabled();
    /// ```
    pub fn with_disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Sets the keys for this binding from string representations.
    ///
    /// This provides a convenient way to set multiple keys using human-readable
    /// string representations.
    ///
    /// # Arguments
    ///
    /// * `keys` - Array of string key representations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    ///
    /// let binding = Binding::new::<&str>(vec![])
    ///     .with_keys(&["ctrl+s", "f2", "alt+s"]);
    /// ```
    pub fn with_keys(mut self, keys: &[&str]) -> Self {
        self.keys = keys.iter().map(|k| parse_key_string(k)).collect();
        self
    }

    /// Sets the keys for the keybinding (mutable version).
    ///
    /// # Arguments
    ///
    /// * `keys` - A vector of items that can be converted to `KeyPress`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let mut binding = Binding::new::<&str>(vec![]);
    /// binding.set_keys(vec![KeyCode::Enter, KeyCode::Char(' ')]);
    /// ```
    pub fn set_keys<K: Into<KeyPress>>(&mut self, keys: Vec<K>) {
        self.keys = keys.into_iter().map(Into::into).collect();
    }

    /// Returns the key presses associated with this binding.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let binding = Binding::new(vec![KeyCode::Enter]);
    /// let keys = binding.keys();
    /// assert_eq!(keys.len(), 1);
    /// ```
    pub fn keys(&self) -> &[KeyPress] {
        &self.keys
    }

    /// Sets the help text for the keybinding (mutable version).
    ///
    /// # Arguments
    ///
    /// * `key` - The human-readable key representation for help display
    /// * `desc` - A brief description of what the binding does
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let mut binding = Binding::new(vec![KeyCode::Enter]);
    /// binding.set_help("enter", "Confirm selection");
    /// ```
    pub fn set_help(&mut self, key: impl Into<String>, desc: impl Into<String>) {
        self.help = Help {
            key: key.into(),
            desc: desc.into(),
        };
    }

    /// Returns the help information for this binding.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let binding = Binding::new(vec![KeyCode::Enter])
    ///     .with_help("enter", "Confirm selection");
    /// let help = binding.help();
    /// assert_eq!(help.key, "enter");
    /// assert_eq!(help.desc, "Confirm selection");
    /// ```
    pub fn help(&self) -> &Help {
        &self.help
    }

    /// Returns `true` if the keybinding is enabled and has keys configured.
    ///
    /// Disabled bindings or bindings with no keys will not match key events
    /// and will not appear in help views.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let binding = Binding::new(vec![KeyCode::Enter]);
    /// assert!(binding.enabled());
    ///
    /// let disabled = Binding::new(vec![KeyCode::F(1)]).with_disabled();
    /// assert!(!disabled.enabled());
    ///
    /// let empty = Binding::new::<KeyCode>(vec![]);
    /// assert!(!empty.enabled());
    /// ```
    pub fn enabled(&self) -> bool {
        self.enabled && !self.keys.is_empty()
    }

    /// Sets the enabled state of the keybinding (mutable version).
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the binding should be enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let mut binding = Binding::new(vec![KeyCode::F(1)]);
    /// binding.set_enabled(false);
    /// assert!(!binding.enabled());
    /// ```
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Removes all keys and help text, effectively nullifying the binding.
    ///
    /// After calling this method, the binding will be disabled and will not
    /// match any key events or appear in help views.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let mut binding = Binding::new(vec![KeyCode::Enter])
    ///     .with_help("enter", "Confirm");
    /// assert!(binding.enabled());
    ///
    /// binding.unbind();
    /// assert!(!binding.enabled());
    /// assert!(binding.keys().is_empty());
    /// ```
    pub fn unbind(&mut self) {
        self.keys.clear();
        self.help = Help::default();
    }

    /// Checks if a `KeyMsg` from `bubbletea-rs` matches this binding.
    ///
    /// The match is successful if the binding is enabled and the event's
    /// `code` and `modifiers` match one of the `KeyPress` entries exactly.
    ///
    /// # Arguments
    ///
    /// * `key_msg` - The key message to test against this binding
    ///
    /// # Returns
    ///
    /// `true` if the key message matches this binding, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use bubbletea_rs::KeyMsg;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let binding = Binding::new(vec![(KeyCode::Char('s'), KeyModifiers::CONTROL)]);
    ///
    /// let ctrl_s = KeyMsg {
    ///     key: KeyCode::Char('s'),
    ///     modifiers: KeyModifiers::CONTROL,
    /// };
    ///
    /// assert!(binding.matches(&ctrl_s));
    /// ```
    pub fn matches(&self, key_msg: &KeyMsg) -> bool {
        if !self.enabled() {
            return false;
        }
        for key_press in &self.keys {
            if key_msg.key == key_press.code && key_msg.modifiers == key_press.mods {
                return true;
            }
        }
        false
    }

    /// A convenience function that checks if a `KeyMsg` matches any of the provided bindings.
    ///
    /// # Arguments
    ///
    /// * `key_msg` - The key message to test
    /// * `bindings` - A slice of binding references to check against
    ///
    /// # Returns
    ///
    /// `true` if the key message matches any of the bindings, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::key::Binding;
    /// use bubbletea_rs::KeyMsg;
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let save = Binding::new(vec![(KeyCode::Char('s'), KeyModifiers::CONTROL)]);
    /// let quit = Binding::new(vec![(KeyCode::Char('q'), KeyModifiers::CONTROL)]);
    ///
    /// let bindings = vec![&save, &quit];
    /// let ctrl_s = KeyMsg {
    ///     key: KeyCode::Char('s'),
    ///     modifiers: KeyModifiers::CONTROL,
    /// };
    ///
    /// assert!(Binding::matches_any(&ctrl_s, &bindings));
    /// ```
    pub fn matches_any(key_msg: &KeyMsg, bindings: &[&Self]) -> bool {
        for binding in bindings {
            if binding.matches(key_msg) {
                return true;
            }
        }
        false
    }
}

/// KeyMap trait for components that provide help information.
///
/// This trait should be implemented by any component that wants to provide
/// contextual help information through the help component. It matches the
/// Go implementation's KeyMap interface.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::{KeyMap, Binding};
/// use crossterm::event::KeyCode;
///
/// struct MyComponent {
///     save: Binding,
///     quit: Binding,
/// }
///
/// impl KeyMap for MyComponent {
///     fn short_help(&self) -> Vec<&Binding> {
///         vec![&self.save, &self.quit]
///     }
///
///     fn full_help(&self) -> Vec<Vec<&Binding>> {
///         vec![
///             vec![&self.save],    // File operations column
///             vec![&self.quit],    // Application column
///         ]
///     }
/// }
/// ```
pub trait KeyMap {
    /// Returns a slice of bindings to be displayed in the short version of help.
    ///
    /// This should return the most important or commonly used key bindings
    /// that will fit on a single line.
    fn short_help(&self) -> Vec<&Binding>;

    /// Returns an extended group of help items, grouped by columns.
    ///
    /// Each inner vector represents a column of related key bindings. This
    /// allows for organized display of comprehensive help information.
    fn full_help(&self) -> Vec<Vec<&Binding>>;
}

/// Checks if the given KeyMsg matches any of the given bindings.
///
/// This is a standalone function similar to Go's Matches function that provides
/// a convenient way to test a key message against multiple bindings at once.
///
/// # Arguments
///
/// * `key_msg` - The key message to test
/// * `bindings` - A slice of binding references to check against
///
/// # Returns
///
/// `true` if the key message matches any of the bindings, `false` otherwise.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::{matches, Binding};
/// use bubbletea_rs::KeyMsg;
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// let save = Binding::new(vec![(KeyCode::Char('s'), KeyModifiers::CONTROL)]);
/// let quit = Binding::new(vec![(KeyCode::Char('q'), KeyModifiers::CONTROL)]);
///
/// let bindings = vec![&save, &quit];
/// let ctrl_s = KeyMsg {
///     key: KeyCode::Char('s'),
///     modifiers: KeyModifiers::CONTROL,
/// };
///
/// assert!(matches(&ctrl_s, &bindings));
/// ```
pub fn matches(key_msg: &KeyMsg, bindings: &[&Binding]) -> bool {
    for binding in bindings {
        if (*binding).matches(key_msg) {
            return true;
        }
    }
    false
}

/// Creates a new binding from options - Go compatibility function.
///
/// This function provides Go-style binding creation using a vector of options.
/// It's equivalent to calling `Binding::new_binding()` but provides a more
/// functional programming style interface.
///
/// # Arguments
///
/// * `opts` - A vector of builder options to configure the binding
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::{new_binding, with_keys_str, with_help};
///
/// let save_binding = new_binding(vec![
///     with_keys_str(&["ctrl+s", "f2"]),
///     with_help("ctrl+s", "Save the current file"),
/// ]);
/// ```
pub fn new_binding(opts: Vec<BindingOpt>) -> Binding {
    Binding::new_binding(opts)
}

/// Creates a binding option that sets the keys from string names.
///
/// This function provides Go-style `WithKeys` functionality, allowing you to
/// set multiple key bindings using human-readable string representations.
///
/// # Arguments
///
/// * `keys` - Array of string key representations
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::{new_binding, with_keys_str, with_help};
///
/// let binding = new_binding(vec![
///     with_keys_str(&["ctrl+s", "alt+s", "f2"]),
///     with_help("ctrl+s", "Save file"),
/// ]);
/// ```
pub fn with_keys_str(keys: &[&str]) -> BindingOpt {
    // Pre-parse into KeyPress so the closure can be 'static
    let parsed: Vec<KeyPress> = keys.iter().map(|s| parse_key_string(s)).collect();
    Box::new(move |b: &mut Binding| {
        b.keys = parsed.clone();
    })
}

/// Checks if a KeyMsg matches a specific binding - Go compatibility.
///
/// This is a convenience function that provides Go-style compatibility for
/// checking if a single binding matches a key message.
///
/// # Arguments
///
/// * `key_msg` - The key message to test
/// * `binding` - The binding to check against
///
/// # Returns
///
/// `true` if the key message matches the binding, `false` otherwise.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::{matches_binding, Binding};
/// use bubbletea_rs::KeyMsg;
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// let binding = Binding::new(vec![(KeyCode::Char('s'), KeyModifiers::CONTROL)]);
/// let ctrl_s = KeyMsg {
///     key: KeyCode::Char('s'),
///     modifiers: KeyModifiers::CONTROL,
/// };
///
/// assert!(matches_binding(&ctrl_s, &binding));
/// ```
pub fn matches_binding(key_msg: &KeyMsg, binding: &Binding) -> bool {
    binding.matches(key_msg)
}

/// Creates a binding option that sets the keys from KeyPress values.
///
/// This function allows you to set key bindings using strongly-typed KeyPress
/// values rather than string representations.
///
/// # Arguments
///
/// * `keys` - A vector of items that can be converted to KeyPress
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::{new_binding, with_keys, with_help};
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// let binding = new_binding(vec![
///     with_keys(vec![
///         (KeyCode::Char('s'), KeyModifiers::CONTROL),
///         (KeyCode::F(2), KeyModifiers::NONE),
///     ]),
///     with_help("ctrl+s", "Save file"),
/// ]);
/// ```
pub fn with_keys<K: Into<KeyPress> + Clone + 'static>(keys: Vec<K>) -> BindingOpt {
    Box::new(move |b: &mut Binding| {
        b.keys = keys.into_iter().map(Into::into).collect();
    })
}

/// Creates a binding option that sets the help text.
///
/// This function provides Go-style `WithHelp` functionality for setting
/// the help text that will be displayed in help views.
///
/// # Arguments
///
/// * `key` - The human-readable key representation for help display
/// * `desc` - A brief description of what the binding does
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::{new_binding, with_keys_str, with_help};
///
/// let binding = new_binding(vec![
///     with_keys_str(&["ctrl+s"]),
///     with_help("ctrl+s", "Save the current file"),
/// ]);
/// ```
pub fn with_help(
    key: impl Into<String> + 'static,
    desc: impl Into<String> + 'static,
) -> BindingOpt {
    Box::new(move |b: &mut Binding| {
        b.help = Help {
            key: key.into(),
            desc: desc.into(),
        };
    })
}

/// Creates a binding option that disables the binding.
///
/// This function provides Go-style `WithDisabled` functionality for creating
/// bindings that are initially disabled.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::{new_binding, with_keys_str, with_disabled};
///
/// let binding = new_binding(vec![
///     with_keys_str(&["f1"]),
///     with_disabled(),
/// ]);
///
/// assert!(!binding.enabled());
/// ```
pub fn with_disabled() -> BindingOpt {
    Box::new(|b: &mut Binding| {
        b.enabled = false;
    })
}

/// Parses string representations of keys into KeyPress instances.
///
/// This function converts human-readable key descriptions into structured
/// KeyPress objects. It supports a wide variety of key formats and combinations.
///
/// # Supported Formats
///
/// ## Simple Keys
/// - Arrow keys: "up", "down", "left", "right"
/// - Special keys: "enter", "tab", "esc"/"escape", "space", "backspace"
/// - Navigation: "home", "end", "pgup"/"pageup", "pgdown"/"pagedown"/"pgdn"
/// - Function keys: "f1" through "f12"
/// - Characters: "a", "1", "?", "/", etc.
///
/// ## Modifier Combinations
/// - Single modifier: "ctrl+c", "alt+f4", "shift+tab"
/// - Double modifiers: "ctrl+alt+a", "ctrl+shift+s"
///
/// # Arguments
///
/// * `s` - The string representation of the key
///
/// # Returns
///
/// A `KeyPress` representing the parsed key combination. Unknown keys
/// result in `KeyCode::Null`.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::key::parse_key_string;
/// use crossterm::event::{KeyCode, KeyModifiers};
///
/// let enter = parse_key_string("enter");
/// let ctrl_c = parse_key_string("ctrl+c");
/// let alt_f4 = parse_key_string("alt+f4");
/// let complex = parse_key_string("ctrl+alt+a");
///
/// assert_eq!(enter.code, KeyCode::Enter);
/// assert_eq!(ctrl_c.code, KeyCode::Char('c'));
/// assert_eq!(ctrl_c.mods, KeyModifiers::CONTROL);
/// ```
///
/// # Panics
///
/// This function does not panic. Invalid or unknown key combinations will
/// result in a KeyPress with `KeyCode::Null`.
pub fn parse_key_string(s: &str) -> KeyPress {
    match s {
        "up" => KeyPress::from(KeyCode::Up),
        "down" => KeyPress::from(KeyCode::Down),
        "left" => KeyPress::from(KeyCode::Left),
        "right" => KeyPress::from(KeyCode::Right),
        "enter" => KeyPress::from(KeyCode::Enter),
        "tab" => KeyPress::from(KeyCode::Tab),
        "backspace" => KeyPress::from(KeyCode::Backspace),
        "delete" | "del" => KeyPress::from(KeyCode::Delete),
        "esc" | "escape" => KeyPress::from(KeyCode::Esc),
        "home" => KeyPress::from(KeyCode::Home),
        "end" => KeyPress::from(KeyCode::End),
        "pgup" | "pageup" => KeyPress::from(KeyCode::PageUp),
        "pgdown" | "pagedown" | "pgdn" => KeyPress::from(KeyCode::PageDown),
        "f1" => KeyPress::from(KeyCode::F(1)),
        "f2" => KeyPress::from(KeyCode::F(2)),
        "f3" => KeyPress::from(KeyCode::F(3)),
        "f4" => KeyPress::from(KeyCode::F(4)),
        "f5" => KeyPress::from(KeyCode::F(5)),
        "f6" => KeyPress::from(KeyCode::F(6)),
        "f7" => KeyPress::from(KeyCode::F(7)),
        "f8" => KeyPress::from(KeyCode::F(8)),
        "f9" => KeyPress::from(KeyCode::F(9)),
        "f10" => KeyPress::from(KeyCode::F(10)),
        "f11" => KeyPress::from(KeyCode::F(11)),
        "f12" => KeyPress::from(KeyCode::F(12)),
        // Handle compound key combinations
        key if key.contains('+') => {
            let parts: Vec<&str> = key.split('+').collect();
            if parts.len() == 2 {
                let (modifier_str, key_str) = (parts[0], parts[1]);
                let modifiers = match modifier_str {
                    "ctrl" => KeyModifiers::CONTROL,
                    "alt" => KeyModifiers::ALT,
                    "shift" => KeyModifiers::SHIFT,
                    _ => KeyModifiers::NONE,
                };

                let keycode = match key_str {
                    "tab" if modifier_str == "shift" => KeyCode::BackTab,
                    "space" | " " => KeyCode::Char(' '),
                    key_str if key_str.len() == 1 => KeyCode::Char(key_str.chars().next().unwrap()),
                    _ => parse_key_string(key_str).code, // Recursive parse for complex keys
                };

                KeyPress::from((keycode, modifiers))
            } else if parts.len() == 3 {
                // Handle ctrl+alt+key combinations
                let key_str = parts[2];
                let mut modifiers = KeyModifiers::NONE;

                for part in &parts[0..2] {
                    match *part {
                        "ctrl" => modifiers |= KeyModifiers::CONTROL,
                        "alt" => modifiers |= KeyModifiers::ALT,
                        "shift" => modifiers |= KeyModifiers::SHIFT,
                        _ => {}
                    }
                }

                let keycode = if key_str.len() == 1 {
                    KeyCode::Char(key_str.chars().next().unwrap())
                } else {
                    parse_key_string(key_str).code
                };

                KeyPress::from((keycode, modifiers))
            } else {
                KeyPress::from(KeyCode::Null)
            }
        }
        " " | "space" => KeyPress::from(KeyCode::Char(' ')),
        "?" => KeyPress::from(KeyCode::Char('?')),
        "/" => KeyPress::from(KeyCode::Char('/')),
        "insert" => KeyPress::from(KeyCode::Insert),
        "null" => KeyPress::from(KeyCode::Null),
        // Single characters
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap();
            KeyPress::from(KeyCode::Char(ch))
        }
        _ => KeyPress::from(KeyCode::Null), // Unknown key
    }
}
