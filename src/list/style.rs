//! Styling system for list components.
//!
//! This module provides comprehensive styling options for list components, including
//! styles for different UI elements and visual states. The styling system is built
//! on top of lipgloss and provides sensible defaults for terminal applications.
//!
//! ## Style Categories
//!
//! - **Title and Header**: Styles for list titles and column headers
//! - **Status and Filter**: Styles for status bar and filter prompt
//! - **Pagination**: Styles for pagination indicators and navigation
//! - **Visual Elements**: Styles for spinners, dots, and separators
//!
//! ## Constants
//!
//! - `BULLET`: Unicode bullet character (`•`) used in pagination
//! - `ELLIPSIS`: Unicode ellipsis character (`…`) for truncated content
//!
//! ## Color Scheme
//!
//! The default styles use a dark theme with:
//! - Bright colors for active/selected elements
//! - Subdued colors for secondary information
//! - High contrast for readability
//!
//! ## Example
//!
//! ```rust
//! use bubbles_rs::list::style::{ListStyles, BULLET, ELLIPSIS};
//! use lipgloss::{Style, Color};
//!
//! let mut styles = ListStyles::default();
//! styles.title = Style::new()
//!     .foreground(Color::from("cyan"))
//!     .bold(true);
//! ```

use lipgloss::{self, style::Style, Color};

/// Unicode bullet character used in pagination indicators.
pub const BULLET: &str = "•";
/// Unicode ellipsis character used for truncated content.
pub const ELLIPSIS: &str = "…";

/// Collection of styles applied to list UI elements.
#[derive(Debug, Clone)]
pub struct ListStyles {
    /// Style for the title bar container.
    pub title_bar: Style,
    /// Style for the list title text.
    pub title: Style,
    /// Style for spinner glyphs.
    pub spinner: Style,
    /// Style for the filter prompt label.
    pub filter_prompt: Style,
    /// Style for the filter cursor/caret.
    pub filter_cursor: Style,
    /// Style for default filter character highlight.
    pub default_filter_character_match: Style,
    /// Style for the status bar container.
    pub status_bar: Style,
    /// Style for the status bar when the list is empty.
    pub status_empty: Style,
    /// Style for active filter text in the status bar.
    pub status_bar_active_filter: Style,
    /// Style for filter match count in the status bar.
    pub status_bar_filter_count: Style,
    /// Style for the "No items" message.
    pub no_items: Style,
    /// Style for pagination area.
    pub pagination_style: Style,
    /// Style for help text area.
    pub help_style: Style,
    /// Style for the active pagination dot.
    pub active_pagination_dot: Style,
    /// Style for the inactive pagination dot.
    pub inactive_pagination_dot: Style,
    /// Style for arabic numerals in pagination.
    pub arabic_pagination: Style,
    /// Style for the divider dot between elements.
    pub divider_dot: Style,
}

impl Default for ListStyles {
    fn default() -> Self {
        // Fallback to dark-theme oriented colors (use dark variants)
        let very_subdued = Color::from("#3C3C3C");
        let subdued = Color::from("#5C5C5C");
        Self {
            title_bar: Style::new().padding(0, 0, 1, 2),
            title: Style::new()
                .background(Color::from("62"))
                .foreground(Color::from("230"))
                .padding(0, 1, 0, 1),
            spinner: Style::new().foreground(Color::from("#747373")),
            filter_prompt: Style::new().foreground(Color::from("#ECFD65")),
            filter_cursor: Style::new().foreground(Color::from("#EE6FF8")),
            default_filter_character_match: Style::new().underline(true),
            status_bar: Style::new()
                .foreground(Color::from("#777777"))
                .padding(0, 0, 1, 2),
            status_empty: Style::new().foreground(subdued.clone()),
            status_bar_active_filter: Style::new().foreground(Color::from("#dddddd")),
            status_bar_filter_count: Style::new().foreground(very_subdued.clone()),
            no_items: Style::new().foreground(Color::from("#626262")),
            arabic_pagination: Style::new().foreground(subdued.clone()),
            pagination_style: Style::new().padding_left(2),
            help_style: Style::new().padding(1, 0, 0, 2),
            active_pagination_dot: Style::new()
                .foreground(Color::from("#979797"))
                .set_string(BULLET),
            inactive_pagination_dot: Style::new()
                .foreground(very_subdued.clone())
                .set_string(BULLET),
            divider_dot: Style::new()
                .foreground(very_subdued)
                .set_string(&format!(" {} ", BULLET)),
        }
    }
}
