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
use lipgloss::{self, style::Style, Color};

/// Applies character-level highlighting to a string based on match indices.
///
/// This function takes a string and a vector of character indices that should be highlighted,
/// then applies the given styles to create highlighted and non-highlighted segments.
///
/// # Arguments
/// * `text` - The text to apply highlighting to
/// * `matches` - Vector of character indices that should be highlighted
/// * `highlight_style` - Style to apply to matched characters
/// * `normal_style` - Style to apply to non-matched characters
///
/// # Returns
/// A styled string with highlighting applied to the specified character positions
fn apply_character_highlighting(
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
    let mut current_pos = 0;

    // Sort match indices to process them in order
    let mut sorted_matches = matches.to_vec();
    sorted_matches.sort_unstable();
    sorted_matches.dedup();

    for &match_idx in &sorted_matches {
        if match_idx >= chars.len() {
            continue;
        }

        // Add any normal characters before this match
        if current_pos < match_idx {
            let segment: String = chars[current_pos..match_idx].iter().collect();
            if !segment.is_empty() {
                result.push_str(&normal_style.render(&segment));
            }
        }

        // Add the highlighted character
        let highlighted_char = chars[match_idx].to_string();
        result.push_str(&highlight_style.render(&highlighted_char));

        current_pos = match_idx + 1;
    }

    // Add any remaining normal characters
    if current_pos < chars.len() {
        let remaining: String = chars[current_pos..].iter().collect();
        if !remaining.is_empty() {
            result.push_str(&normal_style.render(&remaining));
        }
    }

    result
}

/// Styling for the default list item in various states.
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
    fn default() -> Self {
        let normal_title = Style::new()
            .foreground(Color::from("#dddddd"))
            .padding(0, 0, 0, 2);
        let normal_desc = normal_title.clone().foreground(Color::from("#777777"));
        let selected_title = Style::new()
            .border_style(lipgloss::normal_border())
            .border_left(true)
            .border_left_foreground(Color::from("#AD58B4"))
            .foreground(Color::from("#EE6FF8"))
            .padding(0, 0, 0, 1);
        let selected_desc = selected_title.clone().foreground(Color::from("#AD58B4"));
        let dimmed_title = Style::new()
            .foreground(Color::from("#777777"))
            .padding(0, 0, 0, 2);
        let dimmed_desc = dimmed_title.clone().foreground(Color::from("#4D4D4D"));
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

/// Simple item with a title and optional description.
#[derive(Debug, Clone)]
pub struct DefaultItem {
    /// Main item text.
    pub title: String,
    /// Secondary item text (optional display).
    pub desc: String,
}

impl DefaultItem {
    /// Creates a new default item with title and description.
    pub fn new(title: &str, desc: &str) -> Self {
        Self {
            title: title.to_string(),
            desc: desc.to_string(),
        }
    }
}

impl std::fmt::Display for DefaultItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

impl Item for DefaultItem {
    fn filter_value(&self) -> String {
        self.title.clone()
    }
}

/// Delegate that renders `DefaultItem` instances.
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
    pub fn new() -> Self {
        Self::default()
    }
}

impl<I: Item + 'static> ItemDelegate<I> for DefaultDelegate {
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
    fn height(&self) -> usize {
        if self.show_description {
            self.height
        } else {
            1
        }
    }
    fn spacing(&self) -> usize {
        self.spacing
    }
    fn update(&self, _msg: &Msg, _m: &mut Model<I>) -> Option<Cmd> {
        None
    }
}
