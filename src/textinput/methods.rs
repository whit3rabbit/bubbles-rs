//! Core methods for the Model struct.

use super::model::{paste, Model};
use super::types::{EchoMode, PasteErrMsg, PasteMsg, ValidateFunc};
use crate::key::matches;
use bubbletea_rs::{Cmd, KeyMsg, Msg};
use crossterm::event::{KeyCode, KeyModifiers};

impl Model {
    /// Sets the value of the text input.
    ///
    /// This method replaces the entire content of the text input with the provided string.
    /// If a validation function is set, it will be applied to the new value.
    ///
    /// # Arguments
    ///
    /// * `s` - The new string value to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_value("Hello, world!");
    /// assert_eq!(input.value(), "Hello, world!");
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's SetValue method exactly for compatibility.
    pub fn set_value(&mut self, s: &str) {
        let runes: Vec<char> = s.chars().collect();
        let err = self.validate_runes(&runes);
        self.set_value_internal(runes, err);
    }

    /// Internal method to set value with validation
    pub(super) fn set_value_internal(&mut self, runes: Vec<char>, err: Option<String>) {
        self.err = err;

        let empty = self.value.is_empty();

        if self.char_limit > 0 && runes.len() > self.char_limit as usize {
            self.value = runes[..self.char_limit as usize].to_vec();
        } else {
            self.value = runes;
        }

        if (self.pos == 0 && empty) || self.pos > self.value.len() {
            self.set_cursor(self.value.len());
        }

        self.handle_overflow();
        self.update_suggestions();
    }

    /// Returns the current value of the text input.
    ///
    /// # Returns
    ///
    /// A `String` containing the current text value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_value("test");
    /// assert_eq!(input.value(), "test");
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's Value method exactly for compatibility.
    pub fn value(&self) -> String {
        self.value.iter().collect()
    }

    /// Returns the current cursor position as a character index.
    ///
    /// # Returns
    ///
    /// The cursor position as a `usize`, where 0 is the beginning of the text
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_value("hello");
    /// input.set_cursor(2);
    /// assert_eq!(input.position(), 2);
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's Position method exactly for compatibility.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Moves the cursor to the specified position.
    ///
    /// If the position is beyond the end of the text, the cursor will be placed at the end.
    /// This method also handles overflow for horizontal scrolling when the text is wider than the display width.
    ///
    /// # Arguments
    ///
    /// * `pos` - The new cursor position as a character index
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_value("hello world");
    /// input.set_cursor(6); // Position after "hello "
    /// assert_eq!(input.position(), 6);
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's SetCursor method exactly for compatibility.
    pub fn set_cursor(&mut self, pos: usize) {
        self.pos = pos.min(self.value.len());
        self.handle_overflow();
    }

    /// Moves the cursor to the beginning of the input field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_value("hello");
    /// input.cursor_end();
    /// input.cursor_start();
    /// assert_eq!(input.position(), 0);
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's CursorStart method exactly for compatibility.
    pub fn cursor_start(&mut self) {
        self.set_cursor(0);
    }

    /// Moves the cursor to the end of the input field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_value("hello");
    /// input.cursor_start();
    /// input.cursor_end();
    /// assert_eq!(input.position(), 5);
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's CursorEnd method exactly for compatibility.
    pub fn cursor_end(&mut self) {
        self.set_cursor(self.value.len());
    }

    /// Returns whether the text input currently has focus.
    ///
    /// # Returns
    ///
    /// `true` if the input is focused and will respond to key events, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// assert!(!input.focused());
    /// input.focus();
    /// assert!(input.focused());
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's Focused method exactly for compatibility.
    pub fn focused(&self) -> bool {
        self.focus
    }

    /// Sets focus on the text input, enabling it to receive key events.
    ///
    /// When focused, the text input will display a cursor and respond to keyboard input.
    /// This method also focuses the cursor component which may return a command for cursor blinking.
    ///
    /// # Returns
    ///
    /// A `Cmd` that may be used to start cursor blinking animation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// let cmd = input.focus();
    /// assert!(input.focused());
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's Focus method exactly for compatibility.
    pub fn focus(&mut self) -> Cmd {
        self.focus = true;
        self.cursor.focus().unwrap_or_else(|| {
            // If cursor didn't produce a command, return a resolved no-op command
            Box::pin(async { None })
        })
    }

    /// Removes focus from the text input, disabling key event handling.
    ///
    /// When blurred, the text input will not respond to keyboard input and
    /// the cursor will not be visible.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.focus();
    /// assert!(input.focused());
    /// input.blur();
    /// assert!(!input.focused());
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's Blur method exactly for compatibility.
    pub fn blur(&mut self) {
        self.focus = false;
        self.cursor.blur();
    }

    /// Clears all text and resets the cursor to the beginning.
    ///
    /// This method removes all text content and moves the cursor to position 0.
    /// It does not change other settings like placeholder text, validation, or styling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_value("some text");
    /// input.reset();
    /// assert_eq!(input.value(), "");
    /// assert_eq!(input.position(), 0);
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's Reset method exactly for compatibility.
    pub fn reset(&mut self) {
        self.value.clear();
        self.set_cursor(0);
    }

    /// Sets the list of available suggestions for auto-completion.
    ///
    /// Suggestions will be filtered based on the current input and can be navigated
    /// using the configured key bindings (typically up/down arrows and tab to accept).
    ///
    /// # Arguments
    ///
    /// * `suggestions` - A vector of strings that can be suggested to the user
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_suggestions(vec![
    ///     "apple".to_string(),
    ///     "application".to_string(),
    ///     "apply".to_string(),
    /// ]);
    /// input.set_value("app");
    /// // Now suggestions starting with "app" will be available
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's SetSuggestions method exactly for compatibility.
    pub fn set_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions = suggestions
            .into_iter()
            .map(|s| s.chars().collect())
            .collect();
        self.update_suggestions();
    }

    /// Sets the placeholder text displayed when the input is empty.
    ///
    /// # Arguments
    ///
    /// * `placeholder` - The placeholder text to display
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_placeholder("Enter your name...");
    /// // Placeholder will be visible when input is empty and focused
    /// ```
    pub fn set_placeholder(&mut self, placeholder: &str) {
        self.placeholder = placeholder.to_string();
    }

    /// Sets the display width of the input field in characters.
    ///
    /// This controls how many characters are visible at once. If the text is longer
    /// than the width, it will scroll horizontally as the user types or moves the cursor.
    ///
    /// # Arguments
    ///
    /// * `width` - The width in characters. Use 0 for no width limit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_width(20); // Show up to 20 characters at once
    /// ```
    pub fn set_width(&mut self, width: i32) {
        self.width = width;
    }

    /// Sets the echo mode for displaying typed characters.
    ///
    /// # Arguments
    ///
    /// * `mode` - The echo mode to use:
    ///   - `EchoNormal`: Display characters as typed (default)
    ///   - `EchoPassword`: Display asterisks instead of actual characters
    ///   - `EchoNone`: Don't display any characters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::{new, EchoMode};
    ///
    /// let mut input = new();
    /// input.set_echo_mode(EchoMode::EchoPassword);
    /// input.set_value("secret");
    /// // Text will appear as asterisks: ******
    /// ```
    pub fn set_echo_mode(&mut self, mode: EchoMode) {
        self.echo_mode = mode;
    }

    /// Sets the maximum number of characters allowed in the input.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of characters. Use 0 for no limit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_char_limit(10); // Allow up to 10 characters
    /// input.set_value("This is a very long string");
    /// assert_eq!(input.value().len(), 10); // Truncated to 10 characters
    /// ```
    pub fn set_char_limit(&mut self, limit: i32) {
        self.char_limit = limit;
    }

    /// Sets a validation function that will be called whenever the input changes.
    ///
    /// The validation function receives the current input value and should return
    /// `Ok(())` if the input is valid, or `Err(message)` if invalid.
    ///
    /// # Arguments
    ///
    /// * `validate` - A function that takes a `&str` and returns `Result<(), String>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_validate(Box::new(|s: &str| {
    ///     if s.contains('@') {
    ///         Ok(())
    ///     } else {
    ///         Err("Must contain @ symbol".to_string())
    ///     }
    /// }));
    /// ```
    pub fn set_validate(&mut self, validate: ValidateFunc) {
        self.validate = Some(validate);
    }

    /// Processes a message and updates the text input state.
    ///
    /// This method handles keyboard input, cursor movement, text editing operations,
    /// and clipboard operations. It should be called from your application's update loop.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to process (typically a key press or paste event)
    ///
    /// # Returns
    ///
    /// An optional `Cmd` that may need to be executed (e.g., for cursor blinking)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::textinput::new;
    /// use bubbletea_rs::{KeyMsg, Model};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let mut input = new();
    /// input.focus();
    ///
    /// // Simulate typing 'h'
    /// let key_msg = KeyMsg {
    ///     key: KeyCode::Char('h'),
    ///     modifiers: KeyModifiers::NONE,
    /// };
    /// input.update(Box::new(key_msg));
    /// assert_eq!(input.value(), "h");
    /// ```
    pub fn update(&mut self, msg: Msg) -> std::option::Option<Cmd> {
        if !self.focus {
            return std::option::Option::None;
        }

        // Handle key messages
        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            // Check for suggestion acceptance first
            if matches(key_msg, &[&self.key_map.accept_suggestion]) {
                if self.can_accept_suggestion() {
                    let suggestion = &self.matched_suggestions[self.current_suggestion_index];
                    let remaining: Vec<char> = suggestion[self.value.len()..].to_vec();
                    self.value.extend(remaining);
                    self.cursor_end();
                }
                return std::option::Option::None;
            }

            // Handle other key bindings
            if matches(key_msg, &[&self.key_map.delete_word_backward]) {
                self.delete_word_backward();
            } else if matches(key_msg, &[&self.key_map.delete_character_backward]) {
                self.err = None;
                if !self.value.is_empty() && self.pos > 0 {
                    self.value.remove(self.pos - 1);
                    self.pos -= 1;
                    self.err = self.validate_runes(&self.value);
                }
            } else if matches(key_msg, &[&self.key_map.word_backward]) {
                self.word_backward();
            } else if matches(key_msg, &[&self.key_map.character_backward]) {
                if self.pos > 0 {
                    self.set_cursor(self.pos - 1);
                }
            } else if matches(key_msg, &[&self.key_map.word_forward]) {
                self.word_forward();
            } else if matches(key_msg, &[&self.key_map.character_forward]) {
                if self.pos < self.value.len() {
                    self.set_cursor(self.pos + 1);
                }
            } else if matches(key_msg, &[&self.key_map.line_start]) {
                self.cursor_start();
            } else if matches(key_msg, &[&self.key_map.delete_character_forward]) {
                if !self.value.is_empty() && self.pos < self.value.len() {
                    self.value.remove(self.pos);
                    self.err = self.validate_runes(&self.value);
                }
            } else if matches(key_msg, &[&self.key_map.line_end]) {
                self.cursor_end();
            } else if matches(key_msg, &[&self.key_map.delete_after_cursor]) {
                self.delete_after_cursor();
            } else if matches(key_msg, &[&self.key_map.delete_before_cursor]) {
                self.delete_before_cursor();
            } else if matches(key_msg, &[&self.key_map.paste]) {
                return std::option::Option::Some(paste());
            } else if matches(key_msg, &[&self.key_map.delete_word_forward]) {
                self.delete_word_forward();
            } else if matches(key_msg, &[&self.key_map.next_suggestion]) {
                self.next_suggestion();
            } else if matches(key_msg, &[&self.key_map.prev_suggestion]) {
                self.previous_suggestion();
            } else {
                // Regular character input (no Ctrl/Alt modifiers)
                if let KeyCode::Char(ch) = key_msg.key {
                    // Accept when no control/alt modifiers; allow shift (encoded in char case)
                    if !key_msg.modifiers.contains(KeyModifiers::CONTROL)
                        && !key_msg.modifiers.contains(KeyModifiers::ALT)
                    {
                        self.insert_runes_from_user_input(vec![ch]);
                    }
                }
            }

            self.update_suggestions();
        }

        // Handle paste messages
        if let Some(paste_msg) = msg.downcast_ref::<PasteMsg>() {
            let chars: Vec<char> = paste_msg.0.chars().collect();
            self.insert_runes_from_user_input(chars);
        }

        if let Some(paste_err) = msg.downcast_ref::<PasteErrMsg>() {
            self.err = Some(paste_err.0.clone());
        }

        // Update cursor
        let cursor_cmd = self.cursor.update(&msg);

        self.handle_overflow();
        cursor_cmd
    }

    /// Internal method to handle text insertion from user input
    pub(super) fn insert_runes_from_user_input(&mut self, runes: Vec<char>) {
        let mut avail_space = if self.char_limit > 0 {
            let space = self.char_limit - self.value.len() as i32;
            if space <= 0 {
                return;
            }
            Some(space as usize)
        } else {
            None
        };

        // Stuff before and after the cursor
        let mut head = self.value[..self.pos].to_vec();
        let tail = self.value[self.pos..].to_vec();

        // Insert pasted runes
        for r in runes {
            head.push(r);
            self.pos += 1;

            if let Some(ref mut space) = avail_space {
                *space -= 1;
                if *space == 0 {
                    break;
                }
            }
        }

        // Put it all back together
        let mut new_value = head;
        new_value.extend(tail);

        let input_err = self.validate_runes(&new_value);
        self.set_value_internal(new_value, input_err);
    }

    /// Validate the input against the validation function if set
    pub(super) fn validate_runes(&self, runes: &[char]) -> Option<String> {
        if let Some(ref validate) = self.validate {
            let value: String = runes.iter().collect();
            validate(&value).err()
        } else {
            None
        }
    }

    /// Handle overflow for horizontal scrolling viewport
    pub(super) fn handle_overflow(&mut self) {
        if self.width <= 0 {
            self.offset = 0;
            self.offset_right = self.value.len();
            return;
        }

        let value_width = self.value.len();
        if value_width <= self.width as usize {
            self.offset = 0;
            self.offset_right = self.value.len();
            return;
        }

        // Correct right offset if we've deleted characters
        self.offset_right = self.offset_right.min(self.value.len());

        if self.pos < self.offset {
            self.offset = self.pos;
            let mut w = 0;
            let mut i = 0;
            let runes = &self.value[self.offset..];

            while i < runes.len() && w <= self.width as usize {
                w += 1; // Simplified width calculation
                i += 1;
            }

            self.offset_right = self.offset + i;
        } else if self.pos >= self.offset_right {
            self.offset_right = self.pos;

            let mut w = 0;
            let runes = &self.value[..self.offset_right];
            let mut i = runes.len();
            while i > 0 && w < self.width as usize {
                w += 1; // Simplified width calculation
                i = i.saturating_sub(1);
            }
            self.offset = i;
        }
    }
}
