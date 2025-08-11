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
        let mut sections = Vec::new();

        if self.filter_state == FilterState::Filtering {
            // Show filter input interface when actively filtering
            sections.push(format!("Filter: {}", self.filter_input.view()));
        } else if self.show_title {
            let mut title = self.title.clone();

            // When a filter is applied, show the number of matched items
            if self.filter_state == FilterState::FilterApplied {
                let filter_info = format!(" ({} matched)", self.len());
                title = format!("{}{}", title, filter_info);
            }

            // Add styled title
            sections.push(self.styles.title.render(&title));
        }

        // Add status bar in header position (like Go version) when not filtering
        if self.show_status_bar && self.filter_state != FilterState::Filtering {
            let status = self.view_status_line();
            if !status.is_empty() {
                // Apply status bar style with proper padding (matching Go version)
                sections.push(self.styles.status_bar.render(&status));
            }
        }

        sections.join("\n")
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
            return self.styles.no_items.render("No items.");
        }

        // Calculate how many items can fit in the viewport
        let item_height = self.delegate.height() + self.delegate.spacing();
        if item_height == 0 {
            return String::new();
        }

        // Calculate available height for items using the same logic as update_pagination()
        let mut header_height = 0;
        if self.show_title {
            header_height += self.calculate_element_height("title");
        }
        if self.show_status_bar {
            header_height += self.calculate_element_height("status_bar");
        }

        let mut footer_height = 0;
        if self.show_help {
            footer_height += self.calculate_element_height("help");
        }
        if self.show_pagination {
            footer_height += self.calculate_element_height("pagination");
        }

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

    /// Renders the status line for header display (matching Go version layout).
    ///
    /// This creates a simple status line showing item counts that appears in the header
    /// area between the title and items, just like the Go version.
    ///
    /// # Returns
    ///
    /// A formatted string containing just the item count status.
    pub(super) fn view_status_line(&self) -> String {
        let total_items = self.items.len();
        let visible_items = self.len();

        // Simple item count status (like Go version)
        let singular = self.status_item_singular.as_deref().unwrap_or("item");
        let plural = self.status_item_plural.as_deref().unwrap_or("items");
        let noun = if visible_items == 1 { singular } else { plural };

        if total_items == 0 {
            "No items".to_string()
        } else if self.is_empty() && self.filter_state == FilterState::FilterApplied {
            "Nothing matched".to_string()
        } else if self.filter_state == FilterState::FilterApplied {
            // When filtering, show filter query and results like Go
            let query = self.filter_input.value();
            let num_filtered = total_items.saturating_sub(visible_items);
            if !query.is_empty() && num_filtered > 0 {
                format!(
                    "\"{}\" {} {} • {} filtered",
                    query, visible_items, noun, num_filtered
                )
            } else if !query.is_empty() {
                format!("\"{}\" {} {}", query, visible_items, noun)
            } else {
                format!("{} {}", visible_items, noun)
            }
        } else {
            // Normal unfiltered state - simple count
            format!("{} {}", visible_items, noun)
        }
    }

    /// Renders the footer containing only help information (matching Go version layout).
    ///
    /// The footer now only includes help text, as status information has been moved
    /// to the header area to match the Go version's layout.
    ///
    /// The help content automatically adapts to the current filtering state,
    /// showing relevant key bindings for the user's current context.
    ///
    /// # Returns
    ///
    /// A formatted string containing just the help information,
    /// or an empty string if help is disabled.
    pub(super) fn view_footer(&self) -> String {
        if !self.show_help {
            return String::new();
        }

        let help_content = self.help.view(self);
        self.styles.help_style.render(&help_content)
    }
}
