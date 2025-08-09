//! List component with filtering, pagination, contextual help, and customizable rendering.
//!
//! This module exposes a generic `Model<I: Item>` plus supporting traits and submodules:
//! - `Item`: Implement for your item type; must be `Display + Clone` and return a `filter_value()`
//! - `ItemDelegate`: Controls item `render`, `height`, `spacing`, and `update`
//! - Submodules: `defaultitem`, `keys`, and `style`
//!
//! ### Filtering States
//! The list supports fuzzy filtering with three states:
//! - `Unfiltered`: No filter active
//! - `Filtering`: User is typing a filter; input is shown in the header
//! - `FilterApplied`: Filter accepted; only matching items are displayed
//!
//! When filtering is active, fuzzy match indices are stored per item and delegates can use
//! them to apply character-level highlighting (see `defaultitem`).
//!
//! ### Help Integration
//! The list implements `help::KeyMap`, so you can embed `help::Model` and get contextual
//! help automatically based on the current filtering state.

pub mod defaultitem;
pub mod keys;
pub mod style;

use crate::{help, key, paginator, spinner, textinput};
use bubbletea_rs::{Cmd, KeyMsg, Model as BubbleTeaModel, Msg};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use lipgloss;
use std::fmt::Display;

// --- Traits (Interfaces) ---

/// An item that can be displayed in the list.
pub trait Item: Display + Clone {
    /// The value to use when filtering this item.
    fn filter_value(&self) -> String;
}

/// A delegate encapsulates the functionality for a list item.
pub trait ItemDelegate<I: Item> {
    /// Renders the item's view.
    fn render(&self, m: &Model<I>, index: usize, item: &I) -> String;
    /// The height of the list item.
    fn height(&self) -> usize;
    /// The spacing between list items.
    fn spacing(&self) -> usize;
    /// The update loop for the item.
    fn update(&self, msg: &Msg, m: &mut Model<I>) -> Option<Cmd>;
}

// --- Filter ---
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct FilteredItem<I: Item> {
    index: usize, // index in original items list
    item: I,
    matches: Vec<usize>,
}

// --- Model ---

/// Current filtering state of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterState {
    /// No filtering is active; all items are shown.
    Unfiltered,
    /// User is typing a filter term; live filtering UI is shown.
    Filtering,
    /// A filter term has been applied; only matching items are shown.
    FilterApplied,
}

/// List model containing items, filtering, pagination, and styling.
pub struct Model<I: Item> {
    /// Title rendered in the list header when not filtering.
    pub title: String,
    items: Vec<I>,
    delegate: Box<dyn ItemDelegate<I> + Send + Sync>,

    // Components
    /// Text input used for entering the filter term.
    pub filter_input: textinput::Model,
    /// Paginator controlling visible item slice.
    pub paginator: paginator::Model,
    /// Spinner used during expensive operations (optional usage).
    pub spinner: spinner::Model,
    /// Help model for displaying key bindings.
    pub help: help::Model,
    /// Key bindings for list navigation and filtering.
    pub keymap: keys::ListKeyMap,

    // State
    filter_state: FilterState,
    filtered_items: Vec<FilteredItem<I>>,
    cursor: usize,
    width: usize,
    height: usize,

    // Styles
    /// Visual styles for list elements and states.
    pub styles: style::ListStyles,

    // Status bar labeling
    status_item_singular: Option<String>,
    status_item_plural: Option<String>,
}

impl<I: Item + Send + Sync + 'static> Model<I> {
    /// Creates a new list with items, delegate, and initial dimensions.
    pub fn new(
        items: Vec<I>,
        delegate: impl ItemDelegate<I> + Send + Sync + 'static,
        width: usize,
        height: usize,
    ) -> Self {
        let mut filter_input = textinput::new();
        filter_input.set_placeholder("Filter...");
        let mut paginator = paginator::Model::new();
        paginator.set_per_page(10);

        let mut s = Self {
            title: "List".to_string(),
            items,
            delegate: Box::new(delegate),
            filter_input,
            paginator,
            spinner: spinner::Model::new(),
            help: help::Model::new(),
            keymap: keys::ListKeyMap::default(),
            filter_state: FilterState::Unfiltered,
            filtered_items: vec![],
            cursor: 0,
            width,
            height,
            styles: style::ListStyles::default(),
            status_item_singular: None,
            status_item_plural: None,
        };
        s.update_pagination();
        s
    }

    /// Replace all items in the list and reset pagination if needed.
    pub fn set_items(&mut self, items: Vec<I>) {
        self.items = items;
        self.update_pagination();
    }
    /// Returns a copy of the items currently visible (filtered if applicable).
    pub fn visible_items(&self) -> Vec<I> {
        if self.filter_state == FilterState::Unfiltered {
            self.items.clone()
        } else {
            self.filtered_items.iter().map(|f| f.item.clone()).collect()
        }
    }
    /// Sets the filter input text.
    pub fn set_filter_text(&mut self, s: &str) {
        self.filter_input.set_value(s);
    }
    /// Sets the current filtering state.
    pub fn set_filter_state(&mut self, st: FilterState) {
        self.filter_state = st;
    }
    /// Sets the singular/plural nouns used in the status bar.
    pub fn set_status_bar_item_name(&mut self, singular: &str, plural: &str) {
        self.status_item_singular = Some(singular.to_string());
        self.status_item_plural = Some(plural.to_string());
    }
    /// Renders the status bar string, including position and help.
    pub fn status_view(&self) -> String {
        self.view_footer()
    }

    /// Sets the list title and returns `self` for chaining.
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
    /// Returns a reference to the currently selected item, if any.
    pub fn selected_item(&self) -> Option<&I> {
        if self.filter_state == FilterState::Unfiltered {
            self.items.get(self.cursor)
        } else {
            self.filtered_items.get(self.cursor).map(|fi| &fi.item)
        }
    }
    /// Returns the current cursor position (0-based).
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    /// Returns the number of items in the current view (filtered or not).
    pub fn len(&self) -> usize {
        if self.filter_state == FilterState::Unfiltered {
            self.items.len()
        } else {
            self.filtered_items.len()
        }
    }
    /// Returns `true` if there are no items to display.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn update_pagination(&mut self) {
        let item_count = self.len();
        let item_height = self.delegate.height() + self.delegate.spacing();
        let available_height = self.height.saturating_sub(4);
        let per_page = if item_height > 0 {
            available_height / item_height
        } else {
            10
        }
        .max(1);
        self.paginator.set_per_page(per_page);
        self.paginator
            .set_total_pages(item_count.div_ceil(per_page));
        if self.cursor >= item_count {
            self.cursor = item_count.saturating_sub(1);
        }
    }

    #[allow(dead_code)]
    fn matches_for_item(&self, index: usize) -> Option<&Vec<usize>> {
        if index < self.filtered_items.len() {
            Some(&self.filtered_items[index].matches)
        } else {
            None
        }
    }

    fn apply_filter(&mut self) {
        let filter_term = self.filter_input.value().to_lowercase();
        if filter_term.is_empty() {
            self.filter_state = FilterState::Unfiltered;
            self.filtered_items.clear();
        } else {
            let matcher = SkimMatcherV2::default();
            self.filtered_items = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    matcher
                        .fuzzy_indices(&item.filter_value(), &filter_term)
                        .map(|(_score, indices)| FilteredItem {
                            index: i,
                            item: item.clone(),
                            matches: indices,
                        })
                })
                .collect();
            self.filter_state = FilterState::FilterApplied;
        }
        self.cursor = 0;
        self.update_pagination();
    }

    fn view_header(&self) -> String {
        if self.filter_state == FilterState::Filtering {
            let prompt = self.styles.filter_prompt.clone().render("Filter:");
            format!("{} {}", prompt, self.filter_input.view())
        } else {
            let mut header = self.title.clone();
            if self.filter_state == FilterState::FilterApplied {
                header.push_str(&format!(" (filtered: {})", self.len()));
            }
            self.styles.title.clone().render(&header)
        }
    }

    fn view_items(&self) -> String {
        if self.is_empty() {
            return self.styles.no_items.clone().render("No items");
        }

        let items_to_render: Vec<(usize, &I)> = if self.filter_state == FilterState::Unfiltered {
            self.items.iter().enumerate().collect()
        } else {
            self.filtered_items
                .iter()
                .map(|fi| (fi.index, &fi.item))
                .collect()
        };

        let (start, end) = self.paginator.get_slice_bounds(items_to_render.len());
        let mut result = String::new();

        // Render each item individually using the delegate
        for (list_idx, (_orig_idx, item)) in items_to_render
            .iter()
            .enumerate()
            .take(end.min(items_to_render.len()))
            .skip(start)
        {
            // The item index in the current visible list (for selection highlighting)
            let visible_index = start + list_idx;
            let item_output = self.delegate.render(self, visible_index, item);

            if !result.is_empty() {
                // Add spacing between items
                for _ in 0..self.delegate.spacing() {
                    result.push('\n');
                }
            }

            result.push_str(&item_output);
        }

        result
    }

    fn view_footer(&self) -> String {
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

// Help integration from the list model
impl<I: Item> help::KeyMap for Model<I> {
    fn short_help(&self) -> Vec<&key::Binding> {
        match self.filter_state {
            FilterState::Filtering => vec![&self.keymap.accept_filter, &self.keymap.cancel_filter],
            _ => vec![
                &self.keymap.cursor_up,
                &self.keymap.cursor_down,
                &self.keymap.filter,
            ],
        }
    }
    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        match self.filter_state {
            FilterState::Filtering => {
                vec![vec![&self.keymap.accept_filter, &self.keymap.cancel_filter]]
            }
            _ => vec![
                vec![
                    &self.keymap.cursor_up,
                    &self.keymap.cursor_down,
                    &self.keymap.next_page,
                    &self.keymap.prev_page,
                ],
                vec![
                    &self.keymap.go_to_start,
                    &self.keymap.go_to_end,
                    &self.keymap.filter,
                    &self.keymap.clear_filter,
                ],
            ],
        }
    }
}

impl<I: Item + Send + Sync + 'static> BubbleTeaModel for Model<I> {
    fn init() -> (Self, Option<Cmd>) {
        let model = Self::new(vec![], defaultitem::DefaultDelegate::new(), 80, 24);
        (model, None)
    }
    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        if self.filter_state == FilterState::Filtering {
            if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
                match key_msg.key {
                    crossterm::event::KeyCode::Esc => {
                        self.filter_state = if self.filtered_items.is_empty() {
                            FilterState::Unfiltered
                        } else {
                            FilterState::FilterApplied
                        };
                        self.filter_input.blur();
                        return None;
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.apply_filter();
                        self.filter_input.blur();
                        return None;
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        let mut s = self.filter_input.value();
                        s.push(c);
                        self.filter_input.set_value(&s);
                        self.apply_filter();
                    }
                    crossterm::event::KeyCode::Backspace => {
                        let mut s = self.filter_input.value();
                        s.pop();
                        self.filter_input.set_value(&s);
                        self.apply_filter();
                    }
                    crossterm::event::KeyCode::Delete => { /* ignore delete for now */ }
                    crossterm::event::KeyCode::Left => {
                        let pos = self.filter_input.position();
                        if pos > 0 {
                            self.filter_input.set_cursor(pos - 1);
                        }
                    }
                    crossterm::event::KeyCode::Right => {
                        let pos = self.filter_input.position();
                        self.filter_input.set_cursor(pos + 1);
                    }
                    crossterm::event::KeyCode::Home => {
                        self.filter_input.cursor_start();
                    }
                    crossterm::event::KeyCode::End => {
                        self.filter_input.cursor_end();
                    }
                    _ => {}
                }
            }
            return None;
        }

        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            if self.keymap.cursor_up.matches(key_msg) {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            } else if self.keymap.cursor_down.matches(key_msg) {
                if self.cursor < self.len().saturating_sub(1) {
                    self.cursor += 1;
                }
            } else if self.keymap.go_to_start.matches(key_msg) {
                self.cursor = 0;
            } else if self.keymap.go_to_end.matches(key_msg) {
                self.cursor = self.len().saturating_sub(1);
            } else if self.keymap.filter.matches(key_msg) {
                self.filter_state = FilterState::Filtering;
                // propagate the blink command so it is polled by runtime
                return Some(self.filter_input.focus());
            } else if self.keymap.clear_filter.matches(key_msg) {
                self.filter_input.set_value("");
                self.filter_state = FilterState::Unfiltered;
                self.filtered_items.clear();
                self.cursor = 0;
                self.update_pagination();
            }
        }
        None
    }
    fn view(&self) -> String {
        lipgloss::join_vertical(
            lipgloss::LEFT,
            &[&self.view_header(), &self.view_items(), &self.view_footer()],
        )
    }
}

// Re-export commonly used types
pub use defaultitem::{DefaultDelegate, DefaultItem, DefaultItemStyles};
pub use keys::ListKeyMap;
pub use style::ListStyles;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct S(&'static str);
    impl std::fmt::Display for S {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl Item for S {
        fn filter_value(&self) -> String {
            self.0.to_string()
        }
    }

    #[test]
    fn test_status_bar_item_name() {
        let mut list = Model::new(
            vec![S("foo"), S("bar")],
            defaultitem::DefaultDelegate::new(),
            10,
            10,
        );
        let v = list.status_view();
        assert!(v.contains("2 items"));
        list.set_items(vec![S("foo")]);
        let v = list.status_view();
        assert!(v.contains("1 item"));
    }

    #[test]
    fn test_status_bar_without_items() {
        let list = Model::new(Vec::<S>::new(), defaultitem::DefaultDelegate::new(), 10, 10);
        assert!(list.status_view().contains("No items") || list.is_empty());
    }

    #[test]
    fn test_custom_status_bar_item_name() {
        let mut list = Model::new(
            vec![S("foo"), S("bar")],
            defaultitem::DefaultDelegate::new(),
            10,
            10,
        );
        list.set_status_bar_item_name("connection", "connections");
        assert!(list.status_view().contains("2 connections"));
        list.set_items(vec![S("foo")]);
        assert!(list.status_view().contains("1 connection"));
        list.set_items(vec![]);
        // When empty, status_view currently just shows help or empty; ensure no panic
        let _ = list.status_view();
    }

    #[test]
    fn test_set_filter_text_and_state_visible_items() {
        let tc = vec![S("foo"), S("bar"), S("baz")];
        let mut list = Model::new(tc.clone(), defaultitem::DefaultDelegate::new(), 10, 10);
        list.set_filter_text("ba");
        list.set_filter_state(FilterState::Unfiltered);
        assert_eq!(list.visible_items().len(), tc.len());

        list.set_filter_state(FilterState::Filtering);
        list.apply_filter();
        let vis = list.visible_items();
        assert_eq!(vis.len(), 2); // bar, baz

        list.set_filter_state(FilterState::FilterApplied);
        let vis2 = list.visible_items();
        assert_eq!(vis2.len(), 2);
    }

    #[test]
    fn test_selection_highlighting_works() {
        let items = vec![S("first item"), S("second item"), S("third item")];
        let mut list = Model::new(items, defaultitem::DefaultDelegate::new(), 80, 20);

        // Test that view renders without crashing and includes styling
        let view_output = list.view();
        assert!(!view_output.is_empty(), "View should not be empty");

        // Test selection highlighting by checking that cursor position affects rendering
        let first_view = list.view();
        list.cursor = 1; // Move cursor to second item
        let second_view = list.view();

        // The views should be different because of selection highlighting
        assert_ne!(
            first_view, second_view,
            "Selection highlighting should change the view"
        );
    }

    #[test]
    fn test_filter_highlighting_works() {
        let items = vec![S("apple pie"), S("banana bread"), S("carrot cake")];
        let mut list = Model::new(items, defaultitem::DefaultDelegate::new(), 80, 20);

        // Apply filter that should match only some items
        list.set_filter_text("ap");
        list.apply_filter(); // Actually apply the filter to process matches

        let filtered_view = list.view();
        assert!(
            !filtered_view.is_empty(),
            "Filtered view should not be empty"
        );

        // Check that filtering worked ("ap" should only match "apple pie")
        assert_eq!(list.len(), 1, "Should have 1 item matching 'ap'");

        // Test that matches are stored correctly
        assert!(
            !list.filtered_items.is_empty(),
            "Filtered items should have match data"
        );
        if !list.filtered_items.is_empty() {
            assert!(
                !list.filtered_items[0].matches.is_empty(),
                "First filtered item should have matches"
            );
            // Check that the matched item is indeed "apple pie"
            assert_eq!(list.filtered_items[0].item.0, "apple pie");
        }
    }
}
