//! View rendering functions for list components.
//!
//! This module handles all visual rendering aspects of the list including:
//! - Header rendering (title or filter input)
//! - Item rendering with viewport management
//! - Footer rendering (status bar and help)
//! - Complete view composition

use super::types::{FilterState, Item};
use super::Model;

impl<I: Item + Send + Sync + 'static> Model<I> {
    /// Renders the list header based on the current filtering state.
    ///
    /// The header changes based on the current state:
    /// - **Unfiltered**: Shows the list title
    /// - **Filtering**: Shows the filter input interface ("Filter: > ___")
    /// - **FilterApplied**: Shows title with filtered item count ("Title (filtered: N)")
    ///
    /// # Returns
    ///
    /// A styled string containing the appropriate header content.
    pub(super) fn view_header(&self) -> String {
        if self.filter_state == FilterState::Filtering {
            // Show filter input interface when actively filtering
            format!("Filter: {}", self.filter_input.view())
        } else {
            // Show title, optionally with filter status
            let mut header = self.title.clone();
            if self.filter_state == FilterState::FilterApplied {
                header.push_str(&format!(" (filtered: {})", self.len()));
            }
            self.styles.title.clone().render(&header)
        }
    }

    /// Renders the visible items section of the list.
    ///
    /// This method handles the core item rendering logic including:
    /// - Viewport-based item selection for display
    /// - Delegate-based item rendering with original indices
    /// - Empty state handling
    /// - Proper spacing and layout
    ///
    /// ## Viewport Management
    ///
    /// The rendering system uses viewport-based scrolling to show only the items
    /// that fit within the available display space. Items are rendered starting
    /// from `viewport_start` and continuing until the display area is filled or
    /// all items are shown.
    ///
    /// ## Index Semantics
    ///
    /// **CRITICAL**: The delegate's `render` method receives the *original* item index
    /// from the full items list, not a viewport-relative or filtered-relative index.
    /// This design ensures that:
    /// - Cursor highlighting works correctly (`index == m.cursor`)
    /// - Filter highlighting can find matches by searching filtered_items
    /// - Navigation state remains consistent across viewport changes
    ///
    /// ## Empty State Display
    ///
    /// When no items are available (either empty list or no filter matches),
    /// displays an appropriate message styled with the list's no-items style.
    ///
    /// # Returns
    ///
    /// A formatted string containing all visible items with proper styling and spacing.
    pub(super) fn view_items(&self) -> String {
        if self.is_empty() {
            return self.styles.no_items.clone().render("No items.");
        }

        // Calculate how many items can fit in the viewport
        let item_height = self.delegate.height() + self.delegate.spacing();
        if item_height == 0 {
            return String::new();
        }

        // Calculate available height for items
        let header_height = 1; // Title or filter input
        let footer_height = if self.show_status_bar { 2 } else { 0 }; // Status + help
        let available_height = self.height.saturating_sub(header_height + footer_height);
        let max_visible_items = (available_height / item_height).max(1);

        // Determine which items to render based on viewport position
        let items_to_render: Vec<(usize, &I)> = if self.filter_state == FilterState::Unfiltered {
            // For unfiltered lists, use original items with their indices
            self.items
                .iter()
                .enumerate()
                .skip(self.viewport_start)
                .take(max_visible_items)
                .collect()
        } else {
            // For filtered lists, use filtered items but preserve original indices
            self.filtered_items
                .iter()
                .skip(self.viewport_start)
                .take(max_visible_items)
                .map(|fi| (fi.index, &fi.item))
                .collect()
        };

        // Render each visible item using the delegate
        let mut rendered_items = Vec::new();
        for (original_index, item) in items_to_render {
            let rendered = self.delegate.render(self, original_index, item);
            if !rendered.is_empty() {
                rendered_items.push(rendered);

                // Add spacing between items if specified by delegate
                let spacing = self.delegate.spacing();
                for _ in 0..spacing {
                    rendered_items.push(String::new());
                }
            }
        }

        // Join items with newlines, removing any trailing empty lines from spacing
        let mut result = rendered_items.join("\n");
        while result.ends_with('\n') {
            result.pop();
        }

        result
    }

    /// Renders the footer containing status information and help.
    ///
    /// The footer includes:
    /// - Status bar showing current selection and item count (if enabled)
    /// - Contextual help text based on current state and key bindings
    ///
    /// The help content automatically adapts to the current filtering state,
    /// showing relevant key bindings for the user's current context.
    ///
    /// # Returns
    ///
    /// A formatted string containing the status bar and help information,
    /// or an empty string if the status bar is disabled.
    pub(super) fn view_footer(&self) -> String {
        if !self.show_status_bar {
            return String::new();
        }

        let mut footer = String::new();
        if !self.is_empty() {
            let singular = self.status_item_singular.as_deref().unwrap_or("item");
            let plural = self.status_item_plural.as_deref().unwrap_or("items");
            let noun = if self.len() == 1 { singular } else { plural };
            footer.push_str(&format!("{}/{} {}", self.cursor + 1, self.len(), noun));
        }
        let help_view = self.help.view(self);
        if !help_view.is_empty() {
            footer.push('\n');
            footer.push_str(&help_view);
        }
        footer
    }
}
