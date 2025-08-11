//! A paginator component for bubbletea-rs, ported from the Go version.
//!
//! This component is used for calculating pagination and rendering pagination info.
//! Note that this package does not render actual pages of content; it's purely
//! for handling the state and view of the pagination control itself.

use crate::key::{self, KeyMap as KeyMapTrait};
use bubbletea_rs::{KeyMsg, Msg};

/// The type of pagination to display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Type {
    /// Display pagination as Arabic numerals (e.g., "1/5").
    #[default]
    Arabic,
    /// Display pagination as dots (e.g., "● ○ ○ ○ ○").
    Dots,
}

/// Key bindings for different actions within the paginator.
///
/// This structure defines the key bindings that control pagination navigation.
/// It implements the `KeyMap` trait to provide help information for the
/// paginator component.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::paginator::PaginatorKeyMap;
/// use bubbletea_widgets::key;
///
/// let keymap = PaginatorKeyMap::default();
///
/// // Create custom key bindings
/// let custom_keymap = PaginatorKeyMap {
///     prev_page: key::new_binding(vec![
///         key::with_keys_str(&["a", "left"]),
///         key::with_help("a/←", "previous page"),
///     ]),
///     next_page: key::new_binding(vec![
///         key::with_keys_str(&["d", "right"]),
///         key::with_help("d/→", "next page"),
///     ]),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PaginatorKeyMap {
    /// Key binding for navigating to the previous page.
    /// Default keys: PageUp, Left Arrow, 'h'
    pub prev_page: key::Binding,
    /// Key binding for navigating to the next page.
    /// Default keys: PageDown, Right Arrow, 'l'
    pub next_page: key::Binding,
}

impl Default for PaginatorKeyMap {
    /// Creates default key bindings for paginator navigation.
    ///
    /// The default key bindings are:
    /// - **Previous page**: PageUp, Left Arrow, 'h'
    /// - **Next page**: PageDown, Right Arrow, 'l'
    ///
    /// These bindings are commonly used in terminal applications and provide
    /// both arrow key navigation and vim-style 'h'/'l' keys.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::PaginatorKeyMap;
    /// use bubbletea_widgets::key::KeyMap;
    ///
    /// let keymap = PaginatorKeyMap::default();
    /// let help = keymap.short_help();
    /// assert_eq!(help.len(), 2); // prev and next bindings
    /// ```
    fn default() -> Self {
        Self {
            prev_page: key::new_binding(vec![
                key::with_keys_str(&["pgup", "left", "h"]),
                key::with_help("←/h", "prev page"),
            ]),
            next_page: key::new_binding(vec![
                key::with_keys_str(&["pgdown", "right", "l"]),
                key::with_help("→/l", "next page"),
            ]),
        }
    }
}

impl KeyMapTrait for PaginatorKeyMap {
    /// Returns key bindings for the short help view.
    ///
    /// This provides the essential pagination key bindings that will be
    /// displayed in compact help views.
    ///
    /// # Returns
    ///
    /// A vector containing references to the previous page and next page bindings.
    fn short_help(&self) -> Vec<&key::Binding> {
        vec![&self.prev_page, &self.next_page]
    }

    /// Returns key bindings for the full help view.
    ///
    /// This organizes all pagination key bindings into columns for display
    /// in expanded help views. Since pagination only has two keys, they're
    /// grouped together in a single column.
    ///
    /// # Returns
    ///
    /// A vector of vectors, where each inner vector represents a column
    /// of related key bindings.
    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![vec![&self.prev_page, &self.next_page]]
    }
}

/// A paginator model for handling pagination state and rendering.
///
/// This component manages pagination state including current page, total pages,
/// and pagination display style. It can render pagination in two modes:
/// - **Arabic**: Shows page numbers (e.g., "3/10")
/// - **Dots**: Shows dots representing pages (e.g., "○ ○ ● ○ ○")
///
/// The paginator handles key bindings for navigation and provides helper methods
/// for calculating slice bounds and page information.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use bubbletea_widgets::paginator::{Model, Type};
///
/// let mut paginator = Model::new()
///     .with_per_page(10)
///     .with_total_items(150); // Creates 15 pages
///
/// assert_eq!(paginator.total_pages, 15);
/// assert!(paginator.on_first_page());
///
/// paginator.next_page();
/// assert_eq!(paginator.page, 1);
/// ```
///
/// ## Different Display Types
///
/// ```rust
/// use bubbletea_widgets::paginator::{Model, Type};
///
/// let mut paginator = Model::new()
///     .with_total_items(50)
///     .with_per_page(10);
///
/// // Arabic mode (default): "1/5"
/// paginator.paginator_type = Type::Arabic;
/// let arabic_view = paginator.view();
///
/// // Dots mode: "● ○ ○ ○ ○"
/// paginator.paginator_type = Type::Dots;
/// let dots_view = paginator.view();
/// ```
///
/// ## Integration with bubbletea-rs
///
/// ```rust
/// use bubbletea_widgets::paginator::Model as Paginator;
/// use bubbletea_rs::{Model, Cmd, Msg};
///
/// struct App {
///     paginator: Paginator,
///     items: Vec<String>,
/// }
///
/// impl Model for App {
///     fn init() -> (Self, Option<Cmd>) {
///         let items: Vec<String> = (1..=100).map(|i| format!("Item {}", i)).collect();
///         let paginator = Paginator::new()
///             .with_per_page(10)
///             .with_total_items(items.len());
///             
///         (Self { paginator, items }, None)
///     }
///
///     fn update(&mut self, msg: Msg) -> Option<Cmd> {
///         self.paginator.update(&msg);
///         None
///     }
///
///     fn view(&self) -> String {
///         let (start, end) = self.paginator.get_slice_bounds(self.items.len());
///         let page_items: Vec<String> = self.items[start..end].to_vec();
///         
///         format!(
///             "Items:\n{}\n\nPage: {}",
///             page_items.join("\n"),
///             self.paginator.view()
///         )
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Model {
    /// The type of pagination to display (Dots or Arabic).
    pub paginator_type: Type,
    /// The current page.
    pub page: usize,
    /// The number of items per page.
    pub per_page: usize,
    /// The total number of pages.
    pub total_pages: usize,

    /// The character to use for the active page in Dots mode.
    pub active_dot: String,
    /// The character to use for inactive pages in Dots mode.
    pub inactive_dot: String,
    /// The format string for Arabic mode (e.g., "%d/%d").
    pub arabic_format: String,

    /// Key bindings.
    pub keymap: PaginatorKeyMap,
}

impl Default for Model {
    /// Creates a paginator with default settings.
    ///
    /// Default configuration:
    /// - Type: Arabic ("1/5" style)
    /// - Current page: 0 (first page)
    /// - Items per page: 1
    /// - Total pages: 1
    /// - Active dot: "•" (for dots mode)
    /// - Inactive dot: "○" (for dots mode)
    /// - Arabic format: "%d/%d" (current/total)
    /// - Default key bindings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::{Model, Type};
    ///
    /// let paginator = Model::default();
    /// assert_eq!(paginator.paginator_type, Type::Arabic);
    /// assert_eq!(paginator.page, 0);
    /// assert_eq!(paginator.per_page, 1);
    /// assert_eq!(paginator.total_pages, 1);
    /// ```
    fn default() -> Self {
        Self {
            paginator_type: Type::default(),
            page: 0,
            per_page: 1,
            total_pages: 1,
            active_dot: "•".to_string(),
            inactive_dot: "○".to_string(),
            arabic_format: "%d/%d".to_string(),
            keymap: PaginatorKeyMap::default(),
        }
    }
}

impl Model {
    /// Creates a new paginator model with default settings.
    ///
    /// This is equivalent to calling `Model::default()` but provides a more
    /// conventional constructor-style API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let paginator = Model::new();
    /// assert_eq!(paginator.page, 0);
    /// assert_eq!(paginator.total_pages, 1);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the total number of items and calculates total pages (builder pattern).
    ///
    /// This method automatically calculates the total number of pages based on
    /// the total items and the current `per_page` setting. If the current page
    /// becomes out of bounds, it will be adjusted to the last valid page.
    ///
    /// # Arguments
    ///
    /// * `items` - The total number of items to paginate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let paginator = Model::new()
    ///     .with_per_page(10)
    ///     .with_total_items(95); // Will create 10 pages (95/10 = 9.5 -> 10)
    ///
    /// assert_eq!(paginator.total_pages, 10);
    /// ```
    pub fn with_total_items(mut self, items: usize) -> Self {
        self.set_total_items(items);
        self
    }

    /// Sets the number of items per page (builder pattern).
    ///
    /// The minimum value is 1; any value less than 1 will be clamped to 1.
    /// This setting affects how total pages are calculated when using
    /// `set_total_items()` or `with_total_items()`.
    ///
    /// # Arguments
    ///
    /// * `per_page` - Number of items to display per page (minimum 1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let paginator = Model::new()
    ///     .with_per_page(25)
    ///     .with_total_items(100); // Will create 4 pages
    ///
    /// assert_eq!(paginator.per_page, 25);
    /// assert_eq!(paginator.total_pages, 4);
    ///
    /// // Values less than 1 are clamped to 1
    /// let clamped = Model::new().with_per_page(0);
    /// assert_eq!(clamped.per_page, 1);
    /// ```
    pub fn with_per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page.max(1);
        self
    }

    /// Sets the number of items per page (mutable version).
    ///
    /// The minimum value is 1; any value less than 1 will be clamped to 1.
    /// This method modifies the paginator in place.
    ///
    /// # Arguments
    ///
    /// * `per_page` - Number of items to display per page (minimum 1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new();
    /// paginator.set_per_page(15);
    /// assert_eq!(paginator.per_page, 15);
    ///
    /// // Values less than 1 are clamped to 1
    /// paginator.set_per_page(0);
    /// assert_eq!(paginator.per_page, 1);
    /// ```
    pub fn set_per_page(&mut self, per_page: usize) {
        self.per_page = per_page.max(1);
    }

    /// Sets the active dot character for dots mode (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `dot` - The character or styled string to use for the active page
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let paginator = Model::new().with_active_dot("●");
    /// assert_eq!(paginator.active_dot, "●");
    /// ```
    pub fn with_active_dot(mut self, dot: &str) -> Self {
        self.active_dot = dot.to_string();
        self
    }

    /// Sets the inactive dot character for dots mode (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `dot` - The character or styled string to use for inactive pages
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let paginator = Model::new().with_inactive_dot("○");
    /// assert_eq!(paginator.inactive_dot, "○");
    /// ```
    pub fn with_inactive_dot(mut self, dot: &str) -> Self {
        self.inactive_dot = dot.to_string();
        self
    }

    /// Sets the active dot character for dots mode (mutable version).
    ///
    /// # Arguments
    ///
    /// * `dot` - The character or styled string to use for the active page
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new();
    /// paginator.set_active_dot("●");
    /// assert_eq!(paginator.active_dot, "●");
    /// ```
    pub fn set_active_dot(&mut self, dot: &str) {
        self.active_dot = dot.to_string();
    }

    /// Sets the inactive dot character for dots mode (mutable version).
    ///
    /// # Arguments
    ///
    /// * `dot` - The character or styled string to use for inactive pages
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new();
    /// paginator.set_inactive_dot("○");
    /// assert_eq!(paginator.inactive_dot, "○");
    /// ```
    pub fn set_inactive_dot(&mut self, dot: &str) {
        self.inactive_dot = dot.to_string();
    }

    /// Sets the total number of pages directly.
    ///
    /// The minimum value is 1; any value less than 1 will be clamped to 1.
    /// If the current page becomes out of bounds after setting the total pages,
    /// it will be adjusted to the last valid page.
    ///
    /// **Note**: This method sets pages directly. If you want to calculate pages
    /// based on total items, use `set_total_items()` instead.
    ///
    /// # Arguments
    ///
    /// * `pages` - The total number of pages (minimum 1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new();
    /// paginator.set_total_pages(10);
    /// assert_eq!(paginator.total_pages, 10);
    ///
    /// // If current page is out of bounds, it gets adjusted
    /// paginator.page = 15; // Out of bounds
    /// paginator.set_total_pages(5);
    /// assert_eq!(paginator.page, 4); // Adjusted to last page (0-indexed)
    /// ```
    pub fn set_total_pages(&mut self, pages: usize) {
        self.total_pages = pages.max(1);
        // Ensure the current page is not out of bounds
        if self.page >= self.total_pages {
            self.page = self.total_pages.saturating_sub(1);
        }
    }

    /// Calculates and sets the total number of pages based on the total items.
    ///
    /// This method divides the total number of items by the current `per_page`
    /// setting to calculate the total pages. The result is always at least 1,
    /// even for 0 items. If the current page becomes out of bounds after
    /// recalculation, it will be adjusted to the last valid page.
    ///
    /// # Arguments
    ///
    /// * `items` - The total number of items to paginate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new().with_per_page(10);
    ///
    /// // 95 items with 10 per page = 10 pages (95/10 = 9.5 -> 10)
    /// paginator.set_total_items(95);
    /// assert_eq!(paginator.total_pages, 10);
    ///
    /// // 0 items still results in 1 page minimum
    /// paginator.set_total_items(0);
    /// assert_eq!(paginator.total_pages, 1);
    ///
    /// // Exact division
    /// paginator.set_total_items(100);
    /// assert_eq!(paginator.total_pages, 10);
    /// ```
    pub fn set_total_items(&mut self, items: usize) {
        if items == 0 {
            self.total_pages = 1;
        } else {
            self.total_pages = items.div_ceil(self.per_page);
        }

        // Ensure the current page is not out of bounds
        if self.page >= self.total_pages {
            self.page = self.total_pages.saturating_sub(1);
        }
    }

    /// Returns the number of items on the current page.
    ///
    /// This method calculates how many items are actually present on the
    /// current page, which may be less than `per_page` on the last page
    /// or when there are fewer total items than `per_page`.
    ///
    /// # Arguments
    ///
    /// * `total_items` - The total number of items being paginated
    ///
    /// # Returns
    ///
    /// The number of items on the current page, or 0 if there are no items.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new().with_per_page(10);
    ///
    /// // Full page
    /// assert_eq!(paginator.items_on_page(100), 10);
    ///
    /// // Partial last page
    /// paginator.page = 9; // Last page (0-indexed)
    /// assert_eq!(paginator.items_on_page(95), 5); // Only 5 items on page 10
    ///
    /// // No items
    /// assert_eq!(paginator.items_on_page(0), 0);
    /// ```
    pub fn items_on_page(&self, total_items: usize) -> usize {
        if total_items == 0 {
            return 0;
        }
        let (start, end) = self.get_slice_bounds(total_items);
        end - start
    }

    /// Calculates slice bounds for the current page.
    ///
    /// This is a helper function for paginating slices. Given the total length
    /// of your data, it returns the start and end indices for the current page.
    /// The returned bounds can be used directly with slice notation.
    ///
    /// # Arguments
    ///
    /// * `length` - The total length of the data being paginated
    ///
    /// # Returns
    ///
    /// A tuple `(start, end)` where:
    /// - `start` is the inclusive start index for the current page
    /// - `end` is the exclusive end index for the current page
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let items: Vec<i32> = (1..=100).collect();
    /// let mut paginator = Model::new().with_per_page(10);
    ///
    /// // First page (0)
    /// let (start, end) = paginator.get_slice_bounds(items.len());
    /// assert_eq!((start, end), (0, 10));
    /// let page_items = &items[start..end]; // Items 1-10
    ///
    /// // Third page (2)
    /// paginator.page = 2;
    /// let (start, end) = paginator.get_slice_bounds(items.len());
    /// assert_eq!((start, end), (20, 30));
    /// let page_items = &items[start..end]; // Items 21-30
    /// ```
    pub fn get_slice_bounds(&self, length: usize) -> (usize, usize) {
        let start = self.page * self.per_page;
        let end = (start + self.per_page).min(length);
        (start, end)
    }

    /// Returns slice bounds assuming maximum possible data length.
    ///
    /// This is a convenience method that calls `get_slice_bounds()` with
    /// the maximum possible data length (`per_page * total_pages`). It's
    /// useful when you know your data exactly fills the pagination structure.
    ///
    /// # Returns
    ///
    /// A tuple `(start, end)` representing slice bounds for the current page.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new()
    ///     .with_per_page(10)
    ///     .with_total_items(100); // Exactly 10 pages
    ///
    /// paginator.page = 3;
    /// let (start, end) = paginator.start_index_end_index();
    /// assert_eq!((start, end), (30, 40));
    /// ```
    pub fn start_index_end_index(&self) -> (usize, usize) {
        self.get_slice_bounds(self.per_page * self.total_pages)
    }

    /// Navigates to the previous page.
    ///
    /// If the paginator is already on the first page (page 0), this method
    /// has no effect. The page number will not go below 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new().with_per_page(10).with_total_items(100);
    /// paginator.page = 5;
    ///
    /// paginator.prev_page();
    /// assert_eq!(paginator.page, 4);
    ///
    /// // Won't go below 0
    /// paginator.page = 0;
    /// paginator.prev_page();
    /// assert_eq!(paginator.page, 0);
    /// ```
    pub fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }

    /// Navigates to the next page.
    ///
    /// If the paginator is already on the last page, this method has no effect.
    /// The page number will not exceed `total_pages - 1`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new().with_per_page(10).with_total_items(100);
    /// // total_pages = 10, so last page is 9 (0-indexed)
    ///
    /// paginator.page = 5;
    /// paginator.next_page();
    /// assert_eq!(paginator.page, 6);
    ///
    /// // Won't go beyond last page  
    /// paginator.page = 8; // Second to last page
    /// paginator.next_page();
    /// assert_eq!(paginator.page, 9); // Should go to last page (9 is the last valid page)
    /// paginator.next_page();
    /// assert_eq!(paginator.page, 9); // Should stay at last page
    /// ```
    pub fn next_page(&mut self) {
        if !self.on_last_page() {
            self.page += 1;
        }
    }

    /// Returns true if the paginator is on the first page.
    ///
    /// The first page is always page 0 in the 0-indexed pagination system.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new().with_per_page(10).with_total_items(100);
    ///
    /// assert!(paginator.on_first_page());
    ///
    /// paginator.next_page();
    /// assert!(!paginator.on_first_page());
    /// ```
    pub fn on_first_page(&self) -> bool {
        self.page == 0
    }

    /// Returns true if the paginator is on the last page.
    ///
    /// The last page is `total_pages - 1` in the 0-indexed pagination system.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model;
    ///
    /// let mut paginator = Model::new().with_per_page(10).with_total_items(90);
    /// // Creates 9 pages (0-8), so last page is 8
    ///
    /// assert!(!paginator.on_last_page());
    ///
    /// paginator.page = 8; // Last page  
    /// assert!(paginator.on_last_page());
    /// ```
    pub fn on_last_page(&self) -> bool {
        self.page == self.total_pages.saturating_sub(1)
    }

    /// Updates the paginator based on received messages.
    ///
    /// This method should be called from your application's `update()` method
    /// to handle pagination key presses. It automatically responds to the
    /// configured key bindings for next/previous page navigation.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to process, typically containing key press events
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::Model as Paginator;
    /// use bubbletea_rs::{Model, Msg};
    ///
    /// struct App {
    ///     paginator: Paginator,
    /// }
    ///
    /// impl Model for App {
    ///     fn update(&mut self, msg: Msg) -> Option<bubbletea_rs::Cmd> {
    ///         // Forward messages to paginator
    ///         self.paginator.update(&msg);
    ///         None
    ///     }
    ///     
    ///     // ... other methods
    /// #   fn init() -> (Self, Option<bubbletea_rs::Cmd>) { (Self { paginator: Paginator::new() }, None) }
    /// #   fn view(&self) -> String { String::new() }
    /// }
    /// ```
    pub fn update(&mut self, msg: &Msg) {
        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            if self.keymap.next_page.matches(key_msg) {
                self.next_page();
            } else if self.keymap.prev_page.matches(key_msg) {
                self.prev_page();
            }
        }
    }

    /// Renders the paginator as a string.
    ///
    /// The output format depends on the `paginator_type` setting:
    /// - **Arabic**: Shows "current/total" (e.g., "3/10")
    /// - **Dots**: Shows dots with active page highlighted (e.g., "○ ○ ● ○ ○")
    ///
    /// # Returns
    ///
    /// A string representation of the current pagination state.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::paginator::{Model, Type};
    ///
    /// let mut paginator = Model::new().with_per_page(10).with_total_items(50);
    /// // Creates 5 pages, currently on page 0
    ///
    /// // Arabic mode (default)
    /// paginator.paginator_type = Type::Arabic;
    /// assert_eq!(paginator.view(), "1/5"); // 1-indexed for display
    ///
    /// // Dots mode  
    /// paginator.paginator_type = Type::Dots;
    /// assert_eq!(paginator.view(), "• ○ ○ ○ ○");
    ///
    /// // Move to page 2
    /// paginator.page = 2;
    /// assert_eq!(paginator.view(), "○ ○ • ○ ○");
    /// ```
    pub fn view(&self) -> String {
        match self.paginator_type {
            Type::Arabic => self.arabic_view(),
            Type::Dots => self.dots_view(),
        }
    }

    fn arabic_view(&self) -> String {
        self.arabic_format
            .replacen("%d", &(self.page + 1).to_string(), 1)
            .replacen("%d", &self.total_pages.to_string(), 1)
    }

    fn dots_view(&self) -> String {
        let mut s = String::new();
        for i in 0..self.total_pages {
            if i == self.page {
                s.push_str(&self.active_dot);
            } else {
                s.push_str(&self.inactive_dot);
            }
            if i < self.total_pages - 1 {
                s.push(' ');
            }
        }
        s
    }
}
