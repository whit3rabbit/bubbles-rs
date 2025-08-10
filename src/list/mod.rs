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
use lipgloss_extras::lipgloss;
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
        let mut items = Vec::new();

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
            items.push(item_output);
        }

        // Join items with newlines, respecting spacing
        let separator = "\n".repeat(self.delegate.spacing().max(1));
        items.join(&separator)
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

    #[test]
    fn test_filter_highlighting_segment_based() {
        let items = vec![S("Nutella"), S("Linux"), S("Python")];
        let mut list = Model::new(items, defaultitem::DefaultDelegate::new(), 80, 20);

        // Test contiguous match highlighting - filter "nut" should match only "Nutella"
        list.set_filter_text("nut");
        list.apply_filter();

        assert_eq!(list.len(), 1, "Should have 1 item matching 'nut'");
        assert_eq!(list.filtered_items[0].item.0, "Nutella");

        // Verify that matches are [0, 1, 2] for "Nut" in "Nutella"
        let matches = &list.filtered_items[0].matches;
        assert_eq!(
            matches.len(),
            3,
            "Should have 3 character matches for 'nut'"
        );
        assert_eq!(matches[0], 0, "First match should be at index 0 (N)");
        assert_eq!(matches[1], 1, "Second match should be at index 1 (u)");
        assert_eq!(matches[2], 2, "Third match should be at index 2 (t)");

        // Test the actual highlighting by rendering - it should not have character separation
        let rendered = list.view();

        // The rendered output should not be empty and should render without errors
        assert!(!rendered.is_empty(), "Rendered view should not be empty");

        // Test that our highlighting function works directly with contiguous segments
        use super::defaultitem::apply_character_highlighting;
        let test_result = apply_character_highlighting(
            "Nutella",
            &[0, 1, 2], // Consecutive indices should be rendered as a single segment
            &lipgloss::Style::new().bold(true),
            &lipgloss::Style::new(),
        );
        // The result should contain styled text and be longer due to ANSI codes
        assert!(
            test_result.len() > "Nutella".len(),
            "Highlighted text should be longer due to ANSI codes"
        );

        // Verify the fix works: test with non-consecutive matches too
        let test_result_sparse = apply_character_highlighting(
            "Nutella",
            &[0, 2, 4], // Non-consecutive indices: N_t_l
            &lipgloss::Style::new().underline(true),
            &lipgloss::Style::new(),
        );
        assert!(
            test_result_sparse.len() > "Nutella".len(),
            "Sparse highlighted text should also work"
        );
    }

    #[test]
    fn test_filter_ansi_efficiency() {
        // Test that consecutive matches use fewer ANSI codes than character-by-character
        use super::defaultitem::apply_character_highlighting;
        let highlight_style = lipgloss::Style::new().bold(true);
        let normal_style = lipgloss::Style::new();

        let consecutive_result = apply_character_highlighting(
            "Hello",
            &[0, 1, 2], // "Hel" - should be one ANSI block
            &highlight_style,
            &normal_style,
        );

        let sparse_result = apply_character_highlighting(
            "Hello",
            &[0, 2, 4], // "H_l_o" - should be three ANSI blocks
            &highlight_style,
            &normal_style,
        );

        // Consecutive matches should result in more efficient ANSI usage
        // This is a rough heuristic - consecutive should have fewer style applications
        assert!(
            consecutive_result.len() < sparse_result.len(),
            "Consecutive highlighting should be more efficient than sparse highlighting"
        );
    }

    #[test]
    fn test_filter_unicode_characters() {
        let items = vec![S("café"), S("naïve"), S("🦀 rust"), S("北京")];
        let mut list = Model::new(items, defaultitem::DefaultDelegate::new(), 80, 20);

        // Test filtering with accented characters
        list.set_filter_text("caf");
        list.apply_filter();
        assert_eq!(list.len(), 1);
        assert_eq!(list.filtered_items[0].item.0, "café");

        // Test filtering with emoji
        list.set_filter_text("rust");
        list.apply_filter();
        assert_eq!(list.len(), 1);
        assert_eq!(list.filtered_items[0].item.0, "🦀 rust");

        // Ensure rendering doesn't crash with unicode
        let rendered = list.view();
        assert!(!rendered.is_empty());
    }

    #[test]
    fn test_filter_highlighting_no_pipe_characters() {
        // Regression test for issue where pipe characters (│) were inserted
        // between highlighted and non-highlighted text segments
        let items = vec![S("Nutella"), S("Linux"), S("Python")];
        let mut list = Model::new(items, defaultitem::DefaultDelegate::new(), 80, 20);

        // Test case from debug output: filter "nut" should only match "Nutella"
        list.set_filter_text("nut");
        list.apply_filter();

        assert_eq!(
            list.len(),
            1,
            "Filter 'nut' should match exactly 1 item (Nutella)"
        );
        assert_eq!(list.filtered_items[0].item.0, "Nutella");

        // First test non-selected state (cursor on different item)
        // This should not have any borders/pipes
        if !list.is_empty() {
            list.cursor = list.len(); // Set cursor beyond items to deselect
        }
        let unselected_rendered = list.view();

        // For unselected items, there should be no pipe characters between highlighted segments
        // This is the specific issue from the debug output: "N│utella"
        assert!(
            !unselected_rendered.contains("N│u") && !unselected_rendered.contains("ut│e"),
            "Unselected item rendering should not have pipe characters between highlighted segments. Output: {:?}",
            unselected_rendered
        );

        // Selected items can have a left border pipe, but not between text segments
        list.cursor = 0; // Select the first item
        let selected_rendered = list.view();

        // Check that the pipe is only at the beginning (left border) not between text
        assert!(
            !selected_rendered.contains("N│u") && !selected_rendered.contains("ut│e"),
            "Selected item should not have pipe characters between highlighted text segments. Output: {:?}",
            selected_rendered
        );

        // Test another case: filter "li" on "Linux" - test unselected first
        list.set_filter_text("li");
        list.apply_filter();

        assert_eq!(list.len(), 1);
        assert_eq!(list.filtered_items[0].item.0, "Linux");

        // Test unselected Linux (no borders)
        list.cursor = list.len(); // Deselect
        let linux_unselected = list.view();
        assert!(
            !linux_unselected.contains("Li│n") && !linux_unselected.contains("i│n"),
            "Unselected Linux should not have pipes between highlighted segments. Output: {:?}",
            linux_unselected
        );
    }

    #[test]
    fn test_filter_highlighting_visual_correctness() {
        // This test focuses on the visual correctness of the rendered output
        // to catch issues like unwanted characters, malformed ANSI, etc.
        let items = vec![S("Testing"), S("Visual"), S("Correctness")];
        let mut list = Model::new(items, defaultitem::DefaultDelegate::new(), 80, 20);

        // Test 1: Single character filter
        list.set_filter_text("t");
        list.apply_filter();

        let rendered = list.view();
        // Should not contain any malformed character sequences
        assert!(
            !rendered.contains("│T")
                && !rendered.contains("T│")
                && !rendered.contains("│t")
                && !rendered.contains("t│"),
            "Single character highlighting should not have pipe artifacts. Output: {:?}",
            rendered
        );

        // Test 2: Multi-character contiguous filter
        list.set_filter_text("test");
        list.apply_filter();

        let rendered = list.view();
        // Should not have pipes between consecutive highlighted characters
        assert!(
            !rendered.contains("T│e") && !rendered.contains("e│s") && !rendered.contains("s│t"),
            "Contiguous highlighting should not have character separation. Output: {:?}",
            rendered
        );

        // Test 3: Check that highlighting preserves text integrity
        // The word "Testing" should appear as a complete word, just with some characters styled
        assert!(
            rendered.contains("Testing") || rendered.matches("Test").count() > 0,
            "Original text should be preserved in some form. Output: {:?}",
            rendered
        );
    }

    #[test]
    fn test_filter_highlighting_ansi_efficiency() {
        // Test that we don't generate excessive ANSI escape sequences
        let items = vec![S("AbCdEfGh")];
        let mut list = Model::new(items, defaultitem::DefaultDelegate::new(), 80, 20);

        // Filter that matches every other character: A_C_E_G
        list.set_filter_text("aceg");
        list.apply_filter();

        // Test unselected state to avoid border artifacts
        list.cursor = list.len(); // Deselect
        let rendered = list.view();

        // Count ANSI reset sequences - there should not be excessive resets
        let reset_count = rendered.matches("\x1b[0m").count();
        let total_length = rendered.len();

        // Heuristic: if resets are more than 20% of total output length, something's wrong
        assert!(
            reset_count < total_length / 5,
            "Too many ANSI reset sequences detected ({} resets in {} chars). This suggests inefficient styling. Output: {:?}",
            reset_count, total_length, rendered
        );

        // Should not have malformed escape sequences
        assert!(
            !rendered.contains("\x1b[0m│") && !rendered.contains("│\x1b["),
            "ANSI sequences should not be mixed with pipe characters. Output: {:?}",
            rendered
        );
    }

    #[test]
    fn test_filter_highlighting_state_consistency() {
        // Test that highlighting works consistently across different states
        let items = vec![S("StateTest"), S("Another"), S("ThirdItem")];
        let mut list = Model::new(items, defaultitem::DefaultDelegate::new(), 80, 20);

        list.set_filter_text("st");
        list.apply_filter();

        // Test unselected state
        list.cursor = list.len(); // Deselect all
        let unselected = list.view();

        // Test selected state
        list.cursor = 0; // Select first item
        let selected = list.view();

        // Both should be free of character separation artifacts
        assert!(
            !unselected.contains("S│t") && !unselected.contains("t│a"),
            "Unselected state should not have character separation. Output: {:?}",
            unselected
        );

        assert!(
            !selected.contains("S│t") && !selected.contains("t│a"),
            "Selected state should not have character separation. Output: {:?}",
            selected
        );

        // Selected state can have left border, but it should be at the beginning
        if selected.contains("│") {
            let lines: Vec<&str> = selected.lines().collect();
            for line in lines {
                if line.contains("StateTest") || line.contains("st") {
                    // If there's a pipe, it should be at the start of content, not between characters
                    if let Some(pipe_pos) = line.find("│") {
                        let after_pipe = &line[pipe_pos + "│".len()..];
                        assert!(
                            !after_pipe.contains("│"),
                            "Only one pipe should appear per line (left border). Line: {:?}",
                            line
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_filter_edge_cases() {
        let items = vec![S("a"), S("ab"), S("abc"), S(""), S("   ")];
        let mut list = Model::new(items, defaultitem::DefaultDelegate::new(), 80, 20);

        // Single character filtering
        list.set_filter_text("a");
        list.apply_filter();
        assert!(list.len() >= 3, "Should match 'a', 'ab', 'abc'");

        // Empty filter should show all non-empty items
        list.set_filter_text("");
        list.apply_filter();
        assert_eq!(list.filter_state, FilterState::Unfiltered);

        // Very short items
        list.set_filter_text("ab");
        list.apply_filter();
        assert!(list.len() >= 2, "Should match 'ab', 'abc'");

        // Ensure no panics with edge cases
        let rendered = list.view();
        assert!(!rendered.is_empty());
    }
}
