//! Default item implementation and delegate for list components.
//!
//! This module provides the standard item type and delegate implementation for the list component.
//! The `DefaultItem` is a simple item with a title and description, while `DefaultDelegate` handles
//! the rendering and interaction logic for these items.
//!
//! ## Default Item Structure
//!
//! The `DefaultItem` represents a basic list item with:
//! - A title (main text)
//! - A description (secondary text, optional display)
//!
//! ## Default Delegate
//!
//! The `DefaultDelegate` handles:
//! - Rendering items with different visual states (normal, selected, dimmed)
//! - Managing item height and spacing
//! - Filtering and match highlighting (when implemented)
//!
//! ## Styling
//!
//! The `DefaultItemStyles` provides comprehensive styling options:
//! - Normal state styles for title and description
//! - Selected state styles with borders and highlighting
//! - Dimmed state styles for filtered-out items
//! - Filter match highlighting styles
//!
//! ## Example
//!
//! ```rust
//! use bubbletea_widgets::list::{DefaultItem, DefaultDelegate};
//!
//! let item = DefaultItem::new("Task 1", "Complete the documentation");
//! let delegate = DefaultDelegate::new();
//! ```

use super::{Item, ItemDelegate, Model};
use bubbletea_rs::{Cmd, Msg};
use lipgloss_extras::prelude::*;

/// Applies segment-based highlighting to a string based on match indices.
///
/// This function takes a string and a vector of character indices that should be highlighted,
/// then applies the given styles to create highlighted and non-highlighted segments.
/// Unlike character-level highlighting, this groups consecutive match indices into contiguous
/// segments to avoid ANSI escape sequence insertion between individual characters.
///
/// # Arguments
/// * `text` - The text to apply highlighting to
/// * `matches` - Vector of character indices that should be highlighted
/// * `highlight_style` - Style to apply to matched segments
/// * `normal_style` - Style to apply to non-matched segments
///
/// # Returns
/// A styled string with highlighting applied to contiguous segments
pub(super) fn apply_character_highlighting(
    text: &str,
    matches: &[usize],
    highlight_style: &Style,
    normal_style: &Style,
) -> String {
    if matches.is_empty() {
        return normal_style.render(text);
    }

    let chars: Vec<char> = text.chars().collect();
    let mut result = String::new();

    // Sort match indices and remove duplicates
    let mut sorted_matches = matches.to_vec();
    sorted_matches.sort_unstable();
    sorted_matches.dedup();

    // Filter out invalid indices
    let valid_matches: Vec<usize> = sorted_matches
        .into_iter()
        .filter(|&idx| idx < chars.len())
        .collect();

    if valid_matches.is_empty() {
        return normal_style.render(text);
    }

    // Group consecutive indices into segments
    let mut segments: Vec<(usize, usize, bool)> = Vec::new(); // (start, end, is_highlighted)
    let mut current_pos = 0;
    let mut i = 0;

    while i < valid_matches.len() {
        let match_start = valid_matches[i];

        // Add normal segment before this match if needed
        if current_pos < match_start {
            segments.push((current_pos, match_start, false));
        }

        // Find the end of consecutive matches
        let mut match_end = match_start + 1;
        while i + 1 < valid_matches.len() && valid_matches[i + 1] == valid_matches[i] + 1 {
            i += 1;
            match_end = valid_matches[i] + 1;
        }

        // Add highlighted segment
        segments.push((match_start, match_end, true));
        current_pos = match_end;
        i += 1;
    }

    // Add final normal segment if needed
    if current_pos < chars.len() {
        segments.push((current_pos, chars.len(), false));
    }

    // Render each segment with appropriate styling
    for (start, end, is_highlighted) in segments {
        let segment: String = chars[start..end].iter().collect();
        if !segment.is_empty() {
            if is_highlighted {
                result.push_str(&highlight_style.render(&segment));
            } else {
                result.push_str(&normal_style.render(&segment));
            }
        }
    }

    result
}

/// Styling configuration for default list items in various visual states.
///
/// This struct provides comprehensive styling options for rendering list items
/// in different states: normal, selected, and dimmed. Each state can have
/// different styles for both the title and description text.
///
/// The styling system uses adaptive colors that automatically adjust to the
/// terminal's light or dark theme, ensuring optimal readability in any environment.
///
/// # Visual States
///
/// - **Normal**: Default appearance for unselected items
/// - **Selected**: Highlighted appearance with left border for the current selection
/// - **Dimmed**: Faded appearance used during filtering when filter input is empty
///
/// # Theme Adaptation
///
/// All colors use `AdaptiveColor` which automatically switches between light and
/// dark variants based on the terminal's background color detection.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::list::DefaultItemStyles;
/// use lipgloss_extras::prelude::*;
///
/// // Use default styles with adaptive colors
/// let styles = DefaultItemStyles::default();
///
/// // Customize specific styles
/// let mut custom_styles = DefaultItemStyles::default();
/// custom_styles.normal_title = Style::new()
///     .foreground(AdaptiveColor { Light: "#333333", Dark: "#FFFFFF" })
///     .bold(true);
/// ```
#[derive(Debug, Clone)]
pub struct DefaultItemStyles {
    /// Title style in normal (unselected) state.
    pub normal_title: Style,
    /// Description style in normal (unselected) state.
    pub normal_desc: Style,
    /// Title style when the item is selected.
    pub selected_title: Style,
    /// Description style when the item is selected.
    pub selected_desc: Style,
    /// Title style when the item is dimmed (e.g., during filtering).
    pub dimmed_title: Style,
    /// Description style when the item is dimmed.
    pub dimmed_desc: Style,
    /// Style used to highlight filter matches.
    pub filter_match: Style,
}

impl Default for DefaultItemStyles {
    /// Creates default styling that matches the Go bubbles library appearance.
    ///
    /// The default styles provide a clean, professional appearance with:
    /// - Adaptive colors that work in both light and dark terminals
    /// - Left border highlighting for selected items
    /// - Consistent padding and typography
    /// - Subtle dimming for filtered states
    ///
    /// # Theme Colors
    ///
    /// - **Normal text**: Dark text on light backgrounds, light text on dark backgrounds
    /// - **Selected items**: Purple accent with left border
    /// - **Dimmed items**: Muted colors during filtering
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::list::DefaultItemStyles;
    ///
    /// let styles = DefaultItemStyles::default();
    /// // Styles are now ready to use with adaptive colors
    /// ```
    fn default() -> Self {
        let normal_title = Style::new()
            .foreground(AdaptiveColor {
                Light: "#1a1a1a",
                Dark: "#dddddd",
            })
            .padding(0, 0, 0, 2);
        let normal_desc = Style::new()
            .foreground(AdaptiveColor {
                Light: "#A49FA5",
                Dark: "#777777",
            })
            .padding(0, 0, 0, 2);
        let selected_title = Style::new()
            .border_style(normal_border())
            .border_top(false)
            .border_right(false)
            .border_bottom(false)
            .border_left(true)
            .border_left_foreground(AdaptiveColor {
                Light: "#F793FF",
                Dark: "#AD58B4",
            })
            .foreground(AdaptiveColor {
                Light: "#EE6FF8",
                Dark: "#EE6FF8",
            })
            .padding(0, 0, 0, 1);
        let selected_desc = selected_title.clone().foreground(AdaptiveColor {
            Light: "#F793FF",
            Dark: "#AD58B4",
        });
        let dimmed_title = Style::new()
            .foreground(AdaptiveColor {
                Light: "#A49FA5",
                Dark: "#777777",
            })
            .padding(0, 0, 0, 2);
        let dimmed_desc = Style::new()
            .foreground(AdaptiveColor {
                Light: "#C2B8C2",
                Dark: "#4D4D4D",
            })
            .padding(0, 0, 0, 2);
        let filter_match = Style::new().underline(true);
        Self {
            normal_title,
            normal_desc,
            selected_title,
            selected_desc,
            dimmed_title,
            dimmed_desc,
            filter_match,
        }
    }
}

/// A simple list item with title and description text.
///
/// This struct represents a basic list item that can be used with the `DefaultDelegate`
/// for rendering in list components. It provides a straightforward implementation of the
/// `Item` trait with built-in support for filtering and display formatting.
///
/// # Structure
///
/// Each `DefaultItem` contains:
/// - A **title**: The primary text displayed prominently
/// - A **description**: Secondary text shown below the title (when enabled)
///
/// Both fields are always present but the description display can be controlled
/// by the delegate's `show_description` setting.
///
/// # Usage
///
/// `DefaultItem` is designed to work seamlessly with `DefaultDelegate` and provides
/// sensible defaults for most list use cases. For more complex item types with
/// custom data, implement the `Item` trait directly.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::list::DefaultItem;
///
/// // Create a simple item
/// let item = DefaultItem::new("Task 1", "Complete the documentation");
/// println!("{}", item); // Displays: "Task 1"
///
/// // Create items for a todo list
/// let todos = vec![
///     DefaultItem::new("Buy groceries", "Milk, bread, eggs"),
///     DefaultItem::new("Write code", "Implement the new feature"),
///     DefaultItem::new("Review PRs", "Check team submissions"),
/// ];
/// ```
#[derive(Debug, Clone)]
pub struct DefaultItem {
    /// Main item text.
    pub title: String,
    /// Secondary item text (optional display).
    pub desc: String,
}

impl DefaultItem {
    /// Creates a new default item with the specified title and description.
    ///
    /// This constructor creates a new `DefaultItem` with the provided title and description
    /// text. Both parameters are converted to owned `String` values for storage.
    ///
    /// # Arguments
    ///
    /// * `title` - The primary text to display for this item. This will be shown
    ///   prominently and is used for filtering operations.
    /// * `desc` - The secondary descriptive text. This provides additional context
    ///   and is displayed below the title when `show_description` is enabled.
    ///
    /// # Returns
    ///
    /// A new `DefaultItem` instance with the specified title and description.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::list::DefaultItem;
    ///
    /// // Create a task item
    /// let task = DefaultItem::new("Review code", "Check the pull request from yesterday");
    ///
    /// // Create a menu item
    /// let menu_item = DefaultItem::new("Settings", "Configure application preferences");
    ///
    /// // Create an item with empty description
    /// let simple_item = DefaultItem::new("Simple Item", "");
    /// ```
    pub fn new(title: &str, desc: &str) -> Self {
        Self {
            title: title.to_string(),
            desc: desc.to_string(),
        }
    }
}

impl std::fmt::Display for DefaultItem {
    /// Formats the item for display, showing only the title.
    ///
    /// This implementation provides a string representation of the item
    /// using only the title field. The description is not included in
    /// the display output, following the pattern where descriptions are
    /// shown separately in list rendering.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::list::DefaultItem;
    ///
    /// let item = DefaultItem::new("My Task", "This is a description");
    /// assert_eq!(format!("{}", item), "My Task");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl Item for DefaultItem {
    /// Returns the text used for filtering this item.
    ///
    /// This implementation returns the item's title, which means that
    /// filtering operations will search and match against the title text.
    /// The description is not included in filtering to keep the search
    /// focused on the primary item identifier.
    ///
    /// # Returns
    ///
    /// A clone of the item's title string that will be used for fuzzy
    /// matching during filtering operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::list::{DefaultItem, Item};
    ///
    /// let item = DefaultItem::new("Buy groceries", "Milk, bread, eggs");
    /// assert_eq!(item.filter_value(), "Buy groceries");
    ///
    /// // The filter will match against "Buy groceries", not the description
    /// ```
    fn filter_value(&self) -> String {
        self.title.clone()
    }
}

/// A delegate for rendering `DefaultItem` instances in list components.
///
/// This delegate provides the standard rendering logic for `DefaultItem` objects,
/// handling different visual states, filtering highlights, and layout options.
/// It implements the `ItemDelegate` trait to integrate seamlessly with the list
/// component system.
///
/// # Features
///
/// - **Adaptive styling**: Automatically adjusts colors for light/dark terminals
/// - **State rendering**: Handles normal, selected, and dimmed visual states
/// - **Filter highlighting**: Character-level highlighting of search matches
/// - **Flexible layout**: Configurable description display and item spacing
/// - **Responsive design**: Adjusts rendering based on available width
///
/// # Configuration
///
/// The delegate can be customized through its public fields:
/// - `show_description`: Controls whether descriptions are rendered below titles
/// - `styles`: Complete styling configuration for all visual states
///
/// # Usage with List
///
/// The delegate is designed to work with the `Model<DefaultItem>` list component
/// and handles all the rendering complexity automatically.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::list::{DefaultDelegate, DefaultItem, Model};
///
/// // Create a delegate with default settings
/// let delegate = DefaultDelegate::new();
///
/// // Customize the delegate
/// let mut custom_delegate = DefaultDelegate::new();
/// custom_delegate.show_description = false; // Hide descriptions
///
/// // Use with a list
/// let items = vec![
///     DefaultItem::new("Task 1", "First task description"),
///     DefaultItem::new("Task 2", "Second task description"),
/// ];
/// let list = Model::new(items, delegate, 80, 24);
/// ```
#[derive(Debug, Clone)]
pub struct DefaultDelegate {
    /// Whether to show the description beneath the title.
    pub show_description: bool,
    /// Styling used for different visual states.
    pub styles: DefaultItemStyles,
    height: usize,
    spacing: usize,
}

impl Default for DefaultDelegate {
    /// Creates a new delegate with default configuration.
    ///
    /// The default delegate is configured with:
    /// - Description display enabled
    /// - Standard adaptive styling  
    /// - Height of 2 lines (title + description)
    /// - 1 line spacing between items
    ///
    /// This configuration provides a standard list appearance that matches
    /// the Go bubbles library defaults.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::list::DefaultDelegate;
    ///
    /// let delegate = DefaultDelegate::default();
    /// assert_eq!(delegate.show_description, true);
    /// ```
    fn default() -> Self {
        Self {
            show_description: true,
            styles: Default::default(),
            height: 2,
            spacing: 1,
        }
    }
}
impl DefaultDelegate {
    /// Creates a new delegate with default styles and layout.
    ///
    /// This is equivalent to `DefaultDelegate::default()` and provides a convenient
    /// constructor for creating a new delegate with standard settings.
    ///
    /// # Returns
    ///
    /// A new `DefaultDelegate` configured with default settings suitable for
    /// most list use cases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::list::{DefaultDelegate, DefaultItem, Model};
    ///
    /// let delegate = DefaultDelegate::new();
    /// let items = vec![DefaultItem::new("Item 1", "Description 1")];
    /// let list = Model::new(items, delegate, 80, 24);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
}

impl<I: Item + 'static> ItemDelegate<I> for DefaultDelegate {
    /// Renders an item as a styled string for display in the list.
    ///
    /// This method handles the complete rendering pipeline for list items, including:
    /// - State detection (normal, selected, dimmed)
    /// - Filter highlighting with character-level precision
    /// - Style application based on current state
    /// - Layout formatting (single-line or with description)
    ///
    /// The rendering adapts to the current list state, applying different styles
    /// for selected items, dimmed items during filtering, and highlighting
    /// characters that match the current filter.
    ///
    /// # Arguments
    ///
    /// * `m` - The list model containing state information
    /// * `index` - The index of this item in the list
    /// * `item` - The item to render
    ///
    /// # Returns
    ///
    /// A formatted string with ANSI styling codes that represents the visual
    /// appearance of the item. Returns an empty string if the list width is 0.
    ///
    /// # Visual States
    ///
    /// - **Normal**: Standard appearance for unselected items
    /// - **Selected**: Highlighted with left border and accent colors
    /// - **Dimmed**: Faded appearance when filtering with empty input
    /// - **Filtered**: Normal or selected appearance with character highlights
    fn render(&self, m: &Model<I>, index: usize, item: &I) -> String {
        let title = item.to_string();
        let desc = if let Some(di) = (item as &dyn std::any::Any).downcast_ref::<DefaultItem>() {
            di.desc.clone()
        } else {
            String::new()
        };

        if m.width == 0 {
            return String::new();
        }

        let s = &self.styles;
        let is_selected = index == m.cursor;
        let empty_filter =
            m.filter_state == super::FilterState::Filtering && m.filter_input.value().is_empty();
        let is_filtered = matches!(
            m.filter_state,
            super::FilterState::Filtering | super::FilterState::FilterApplied
        );

        // Get filter matches for this item if filtering is active
        let matches = if is_filtered && index < m.filtered_items.len() {
            Some(&m.filtered_items[index].matches)
        } else {
            None
        };

        let mut title_out = title.clone();
        let mut desc_out = desc.clone();

        if empty_filter {
            title_out = s.dimmed_title.clone().render(&title_out);
            desc_out = s.dimmed_desc.clone().render(&desc_out);
        } else if is_selected && m.filter_state != super::FilterState::Filtering {
            // Apply highlighting for selected items
            if let Some(match_indices) = matches {
                let highlight_style = s.selected_title.clone().inherit(s.filter_match.clone());
                title_out = apply_character_highlighting(
                    &title,
                    match_indices,
                    &highlight_style,
                    &s.selected_title,
                );
                if !desc.is_empty() {
                    let desc_highlight_style =
                        s.selected_desc.clone().inherit(s.filter_match.clone());
                    desc_out = apply_character_highlighting(
                        &desc,
                        match_indices,
                        &desc_highlight_style,
                        &s.selected_desc,
                    );
                }
            } else {
                title_out = s.selected_title.clone().render(&title_out);
                desc_out = s.selected_desc.clone().render(&desc_out);
            }
        } else {
            // Apply highlighting for normal (unselected) items
            if let Some(match_indices) = matches {
                let highlight_style = s.normal_title.clone().inherit(s.filter_match.clone());
                title_out = apply_character_highlighting(
                    &title,
                    match_indices,
                    &highlight_style,
                    &s.normal_title,
                );
                if !desc.is_empty() {
                    let desc_highlight_style =
                        s.normal_desc.clone().inherit(s.filter_match.clone());
                    desc_out = apply_character_highlighting(
                        &desc,
                        match_indices,
                        &desc_highlight_style,
                        &s.normal_desc,
                    );
                }
            } else {
                title_out = s.normal_title.clone().render(&title_out);
                desc_out = s.normal_desc.clone().render(&desc_out);
            }
        }

        if self.show_description && !desc_out.is_empty() {
            format!("{}\n{}", title_out, desc_out)
        } else {
            title_out
        }
    }
    /// Returns the height in lines that each item occupies.
    ///
    /// The height depends on whether descriptions are enabled:
    /// - With descriptions: Returns the configured height (default 2 lines)
    /// - Without descriptions: Always returns 1 line
    ///
    /// This height is used by the list component for layout calculations,
    /// viewport sizing, and scroll positioning.
    ///
    /// # Returns
    ///
    /// The number of terminal lines each item will occupy when rendered.
    fn height(&self) -> usize {
        if self.show_description {
            self.height
        } else {
            1
        }
    }
    /// Returns the number of blank lines between items.
    ///
    /// This spacing is added between each item in the list to improve
    /// readability and visual separation. The default spacing is 1 line.
    ///
    /// # Returns
    ///
    /// The number of blank lines to insert between rendered items.
    fn spacing(&self) -> usize {
        self.spacing
    }
    /// Handles update messages for the delegate.
    ///
    /// The default delegate implementation does not require any message handling,
    /// so this method always returns `None`. Override this method in custom
    /// delegates that need to respond to keyboard input, timer events, or
    /// other application messages.
    ///
    /// # Arguments
    ///
    /// * `_msg` - The message to handle (unused in default implementation)
    /// * `_m` - Mutable reference to the list model (unused in default implementation)
    ///
    /// # Returns
    ///
    /// Always returns `None` as the default delegate requires no update commands.
    fn update(&self, _msg: &Msg, _m: &mut Model<I>) -> Option<Cmd> {
        None
    }
}
