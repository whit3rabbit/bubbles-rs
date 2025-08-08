//! Suggestion system methods for the textinput component.

use super::model::Model;

impl Model {
    /// Check if a suggestion can be accepted
    pub(super) fn can_accept_suggestion(&self) -> bool {
        !self.matched_suggestions.is_empty()
    }

    /// Update the list of matched suggestions based on current input
    pub(super) fn update_suggestions(&mut self) {
        if self.value.is_empty() || self.suggestions.is_empty() {
            self.matched_suggestions.clear();
            return;
        }

        let current_text: String = self.value.iter().collect();
        let current_lower = current_text.to_lowercase();

        let matches: Vec<Vec<char>> = self
            .suggestions
            .iter()
            .filter(|suggestion| {
                let suggestion_str: String = suggestion.iter().collect();
                suggestion_str.to_lowercase().starts_with(&current_lower)
            })
            .cloned()
            .collect();

        if matches != self.matched_suggestions {
            self.current_suggestion_index = 0;
        }

        self.matched_suggestions = matches;
    }

    /// Move to the next suggestion
    pub(super) fn next_suggestion(&mut self) {
        if !self.matched_suggestions.is_empty() {
            self.current_suggestion_index =
                (self.current_suggestion_index + 1) % self.matched_suggestions.len();
        }
    }

    /// Move to the previous suggestion
    pub(super) fn previous_suggestion(&mut self) {
        if !self.matched_suggestions.is_empty() {
            if self.current_suggestion_index == 0 {
                self.current_suggestion_index = self.matched_suggestions.len() - 1;
            } else {
                self.current_suggestion_index -= 1;
            }
        }
    }

    /// Returns the complete list of available suggestions.
    ///
    /// This returns all suggestions that were set via `set_suggestions()`,
    /// regardless of the current input or filtering.
    ///
    /// # Returns
    ///
    /// A vector of all available suggestion strings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_suggestions(vec!["apple".to_string(), "banana".to_string()]);
    /// let suggestions = input.available_suggestions();
    /// assert_eq!(suggestions.len(), 2);
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's AvailableSuggestions method exactly.
    pub fn available_suggestions(&self) -> Vec<String> {
        self.suggestions
            .iter()
            .map(|s| s.iter().collect())
            .collect()
    }

    /// Returns the list of suggestions that match the current input.
    ///
    /// This returns only the suggestions that start with the current input value
    /// (case-insensitive matching). The list is updated automatically as the user types.
    ///
    /// # Returns
    ///
    /// A vector of suggestion strings that match the current input
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_suggestions(vec![
    ///     "apple".to_string(),
    ///     "application".to_string(),
    ///     "banana".to_string(),
    /// ]);
    /// input.set_value("app");
    /// let matched = input.matched_suggestions();
    /// assert_eq!(matched.len(), 2); // "apple" and "application"
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's MatchedSuggestions method exactly.
    pub fn matched_suggestions(&self) -> Vec<String> {
        self.matched_suggestions
            .iter()
            .map(|s| s.iter().collect())
            .collect()
    }

    /// Returns the index of the currently selected suggestion.
    ///
    /// This index refers to the position in the matched suggestions list
    /// (not the complete available suggestions list). Use up/down arrow keys
    /// or configured navigation bindings to change the selection.
    ///
    /// # Returns
    ///
    /// The zero-based index of the currently selected suggestion
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_suggestions(vec!["apple".to_string(), "application".to_string()]);
    /// input.set_value("app");
    /// assert_eq!(input.current_suggestion_index(), 0); // First suggestion selected
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's CurrentSuggestionIndex method exactly.
    pub fn current_suggestion_index(&self) -> usize {
        self.current_suggestion_index
    }

    /// Returns the text of the currently selected suggestion.
    ///
    /// If no suggestions are available or the index is out of bounds,
    /// this returns an empty string.
    ///
    /// # Returns
    ///
    /// The currently selected suggestion as a `String`, or empty string if none
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::textinput::new;
    ///
    /// let mut input = new();
    /// input.set_suggestions(vec!["apple".to_string(), "application".to_string()]);
    /// input.set_value("app");
    /// assert_eq!(input.current_suggestion(), "apple");
    /// ```
    ///
    /// # Note
    ///
    /// This method matches Go's CurrentSuggestion method exactly.
    pub fn current_suggestion(&self) -> String {
        if self.current_suggestion_index >= self.matched_suggestions.len() {
            return String::new();
        }
        self.matched_suggestions[self.current_suggestion_index]
            .iter()
            .collect()
    }
}
