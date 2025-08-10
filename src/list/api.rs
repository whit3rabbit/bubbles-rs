//! Enhanced API methods for list filter state management.
//!
//! This module provides convenient methods for programmatic filter state management,
//! eliminating the need for workarounds or manual state parsing. These methods
//! complement the existing keyboard-driven interface with clean programmatic access.

use super::types::{FilterState, FilterStateInfo, Item};
use super::Model;
use bubbletea_rs::Cmd;

impl<I: Item + Send + Sync + 'static> Model<I> {
    /// Returns true if filtering is currently active in any form.
    ///
    /// This method provides a simple way to check if the list is in any filtering
    /// state (either actively typing a filter or has an applied filter). It's useful
    /// for applications that need to conditionally show UI elements or change behavior
    /// based on filtering status.
    ///
    /// # Returns
    ///
    /// - `true` if in `Filtering` or `FilterApplied` state
    /// - `false` if in `Unfiltered` state
    ///
    /// # Examples
    ///
    /// ```
    /// use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem, FilterState};
    ///
    /// let items = vec![DefaultItem::new("Apple", "Red fruit")];
    /// let mut list = Model::new(items, DefaultDelegate::new(), 80, 24);
    ///
    /// assert!(!list.is_filtering()); // Initially not filtering
    ///
    /// // Simulate applying a filter (would normally be done through user input)
    /// list.set_filter_text("app");
    /// // In a real application, filter would be applied through the update() method
    /// assert!(!list.is_filtering()); // Still not filtering until state changes
    /// ```
    pub fn is_filtering(&self) -> bool {
        matches!(
            self.filter_state,
            FilterState::Filtering | FilterState::FilterApplied
        )
    }

    /// Forces complete filter clearing in a single operation.
    ///
    /// This method provides a programmatic way to completely clear any active filter,
    /// equivalent to the user pressing the clear filter key binding. It's useful for
    /// applications that need to reset the list state or implement custom clear
    /// functionality.
    ///
    /// # Returns
    ///
    /// Returns `None` as no follow-up commands are needed for the clear operation.
    ///
    /// # Effects
    ///
    /// - Clears the filter input text
    /// - Sets state to `Unfiltered`
    /// - Clears filtered items list
    /// - Resets cursor to position 0
    /// - Updates pagination
    ///
    /// # Examples
    ///
    /// ```
    /// use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    ///
    /// let items = vec![DefaultItem::new("Apple", "Red fruit")];
    /// let mut list = Model::new(items, DefaultDelegate::new(), 80, 24);
    ///
    /// // Apply a filter (this would normally be done through user interaction)
    /// list.set_filter_text("app");
    ///
    /// // Clear it programmatically
    /// let cmd = list.clear_filter();
    /// assert!(cmd.is_none()); // Returns None
    /// assert!(!list.is_filtering()); // No longer filtering
    /// ```
    pub fn clear_filter(&mut self) -> Option<Cmd> {
        self.filter_input.set_value("");
        self.filter_state = FilterState::Unfiltered;
        self.filtered_items.clear();
        self.cursor = 0;
        self.update_pagination();
        None
    }

    /// Returns detailed information about the current filter state.
    ///
    /// This method provides comprehensive information about filtering without requiring
    /// direct access to internal fields. It's particularly useful for applications that
    /// need to display filter status information or make decisions based on detailed
    /// filter state.
    ///
    /// # Returns
    ///
    /// A `FilterStateInfo` struct containing:
    /// - Current filter state enum
    /// - Filter query text
    /// - Number of matching items
    /// - Whether filtering is active
    /// - Whether currently in clearing state
    ///
    /// # Examples
    ///
    /// ```
    /// use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem, FilterState};
    ///
    /// let items = vec![
    ///     DefaultItem::new("Apple", "Red fruit"),
    ///     DefaultItem::new("Banana", "Yellow fruit"),
    /// ];
    /// let mut list = Model::new(items, DefaultDelegate::new(), 80, 24);
    ///
    /// // Check initial state
    /// let info = list.filter_state_info();
    /// assert_eq!(info.state, FilterState::Unfiltered);
    /// assert_eq!(info.query, "");
    /// assert_eq!(info.match_count, 2); // All items visible
    /// assert!(!info.is_filtering);
    /// assert!(!info.is_clearing);
    ///
    /// // Set filter text (would be applied through user interaction)
    /// list.set_filter_text("app");
    /// let info = list.filter_state_info();
    /// assert_eq!(info.query, "app"); // Query is set
    /// ```
    pub fn filter_state_info(&self) -> FilterStateInfo {
        FilterStateInfo {
            state: self.filter_state.clone(),
            query: self.filter_input.value(),
            match_count: self.len(),
            is_filtering: self.is_filtering(),
            is_clearing: false, // This would be true during intermediate clearing states
        }
    }
}
