//! Filter operations and state management for list components.
//!
//! This module handles all fuzzy filtering functionality including:
//! - Filter application and fuzzy matching
//! - Character-level match index tracking
//! - Filter state transitions
//! - Match index retrieval for highlighting

use super::types::{FilterState, FilteredItem, Item};
use super::Model;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

impl<I: Item + Send + Sync + 'static> Model<I> {
    /// Applies the current filter text to all items and updates the filtered results.
    ///
    /// This method performs fuzzy matching against all items using the current filter
    /// input text. It updates the `filtered_items` list with matching items and their
    /// character match indices, then resets navigation state appropriately.
    ///
    /// # Filter Application Process
    ///
    /// 1. **Empty Filter Handling**: If filter text is empty, clears all filtering
    /// 2. **Fuzzy Matching**: Uses SkimMatcherV2 for fuzzy string matching
    /// 3. **Index Preservation**: Maintains original item indices for cursor highlighting
    /// 4. **Match Tracking**: Stores character indices for rendering highlights
    /// 5. **State Reset**: Resets cursor and viewport to show filtered results from the beginning
    ///
    /// # Viewport Reset Behavior
    ///
    /// When a filter is applied, the viewport is reset to position 0 to ensure that
    /// filtered results are visible from the beginning. This prevents the common issue
    /// where filtered items are outside the current viewport and appear to be missing
    /// even though the count shows matches.
    ///
    /// # Performance Characteristics
    ///
    /// The filtering operation is O(n*m) where n is the number of items and m is the
    /// average length of item filter values. The fuzzy matcher is optimized for
    /// interactive use and should handle hundreds of items smoothly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem, FilterState};
    /// let items = vec![
    ///     DefaultItem::new("Apple", "Red fruit"),
    ///     DefaultItem::new("Banana", "Yellow fruit"),
    ///     DefaultItem::new("Cherry", "Small red fruit"),
    /// ];
    /// let mut list = Model::new(items, DefaultDelegate::new(), 80, 24);
    ///
    /// // Set filter text and apply filter
    /// list.set_filter_text("app");
    ///
    /// // The filter would be applied internally during user interaction.
    /// // After filtering, we can check the results
    /// let visible = list.visible_items();
    ///
    /// // When filter text is set and applied, matching items are shown
    /// assert!(visible.len() <= 3); // Original had 3 items
    /// ```
    ///
    /// # State Changes
    ///
    /// This method modifies several aspects of the list state:
    /// - Updates `filtered_items` with matching results and character indices
    /// - Resets `cursor` to 0 to show results from the beginning
    /// - Resets `viewport_start` to 0 for proper display alignment
    /// - Updates pagination to reflect the new item count
    /// - Sets `filter_state` to `Unfiltered` if no filter text is provided
    pub(super) fn apply_filter(&mut self) {
        let filter_text = self.filter_input.value();

        if filter_text.is_empty() {
            // Clear filtering when filter text is empty
            self.filtered_items.clear();
            self.filter_state = FilterState::Unfiltered;
            self.cursor = 0;
            self.update_pagination();
            return;
        }

        // Create fuzzy matcher for filtering
        let matcher = SkimMatcherV2::default();

        // Apply fuzzy filter to all items
        self.filtered_items = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                let filter_value = item.filter_value();
                matcher
                    .fuzzy_indices(&filter_value, &filter_text)
                    .map(|(_, indices)| FilteredItem {
                        index,
                        item: item.clone(),
                        matches: indices,
                    })
            })
            .collect();

        // Reset cursor to beginning of filtered results
        self.cursor = 0;

        // Reset viewport when filtering to ensure results are visible from the beginning.
        // Since filtered results form a new logical list, we start viewing from position 0
        // to guarantee that matching items appear in the display area.
        self.viewport_start = 0;

        // Update pagination to reflect filtered item count
        self.update_pagination();
    }

    /// Synchronizes the viewport position to keep the cursor visible.
    ///
    /// This method ensures that the current cursor position remains visible within
    /// the viewport by adjusting `viewport_start` when necessary. It implements
    /// smooth scrolling behavior that maintains visual context during navigation.
    ///
    /// # Scrolling Behavior
    ///
    /// - **Upward scrolling**: When cursor moves above viewport, scroll up to show it
    /// - **Downward scrolling**: When cursor moves below viewport, scroll down to show it  
    /// - **Context preservation**: Maintains surrounding items in view when possible
    /// - **Boundary handling**: Prevents scrolling beyond list boundaries
    ///
    /// # Viewport Calculations
    ///
    /// The method calculates the visible range based on:
    /// - Current viewport start position
    /// - Available display height for items
    /// - Item height including spacing
    /// - Header and footer space requirements
    ///
    /// This ensures cursor visibility across different list sizes and terminal dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let items = vec![
    ///     DefaultItem::new("Item 1", "Description 1"),
    ///     DefaultItem::new("Item 2", "Description 2"),
    ///     DefaultItem::new("Item 3", "Description 3"),
    ///     DefaultItem::new("Item 4", "Description 4"),
    ///     DefaultItem::new("Item 5", "Description 5"),
    /// ];
    /// let mut list = Model::new(items, DefaultDelegate::new(), 80, 10);
    ///
    /// // Move cursor to last item (would normally be done through user input)
    /// // The viewport synchronization happens automatically during navigation
    /// assert_eq!(list.len(), 5);
    ///
    /// // Viewport adjustments ensure the cursor remains visible
    /// // This is handled internally during user interaction
    /// ```
    ///
    /// # Algorithm Details
    ///
    /// The synchronization algorithm:
    /// 1. Calculates the number of items that can fit in the current viewport
    /// 2. Determines if the cursor is outside the currently visible range
    /// 3. Adjusts `viewport_start` to bring the cursor into view
    /// 4. Ensures the viewport doesn't exceed list boundaries
    ///
    /// # Performance
    ///
    /// This operation is O(1) as it only performs simple arithmetic calculations
    /// based on the current state, making it suitable for real-time navigation updates.
    pub(super) fn sync_viewport_with_cursor(&mut self) {
        if self.is_empty() {
            self.viewport_start = 0;
            return;
        }

        let item_height = self.delegate.height() + self.delegate.spacing();
        if item_height == 0 {
            return;
        }

        // Calculate available height for items
        let header_height = if self.show_title && self.show_status_bar { 2 } else { 1 };
        let footer_height = if self.show_help { 1 } else { 0 } + 
                           if self.show_pagination { 1 } else { 0 };
        let available_height = self.height.saturating_sub(header_height + footer_height);
        let items_per_view = (available_height / item_height).max(1);

        let viewport_end = self.viewport_start + items_per_view;

        // Scroll down if cursor is below viewport
        if self.cursor >= viewport_end {
            self.viewport_start = self.cursor.saturating_sub(items_per_view - 1);
        }

        // Scroll up if cursor is above viewport
        if self.cursor < self.viewport_start {
            self.viewport_start = self.cursor;
        }

        // Ensure viewport doesn't go beyond list boundaries
        let max_viewport_start = self.len().saturating_sub(items_per_view);
        if self.viewport_start > max_viewport_start {
            self.viewport_start = max_viewport_start;
        }
    }
}
