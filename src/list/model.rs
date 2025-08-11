//! Core Model struct and fundamental functionality.
//!
//! This module contains the primary Model struct definition and its core methods
//! including construction, basic state management, and essential accessors.

use super::keys::ListKeyMap;
use super::style::ListStyles;
use super::types::{FilterState, FilteredItem, Item, ItemDelegate};
use crate::{help, paginator, spinner, textinput};

/// A flexible, interactive list component with filtering, pagination, and customizable rendering.
///
/// The `Model<I>` is the main list component that can display any items implementing the `Item` trait.
/// It provides fuzzy filtering, keyboard navigation, viewport scrolling, help integration, and
/// customizable styling through delegates.
///
/// # Features
///
/// - **Fuzzy filtering**: Real-time search with character-level highlighting
/// - **Smooth scrolling**: Viewport-based navigation that maintains context
/// - **Customizable rendering**: Delegate pattern for complete visual control
/// - **Keyboard navigation**: Vim-style keys plus standard arrow navigation
/// - **Contextual help**: Automatic help text generation from key bindings
/// - **Responsive design**: Adapts to different terminal sizes
/// - **State management**: Clean separation of filtering, selection, and view states
///
/// # Architecture
///
/// The list uses a viewport-based scrolling system that maintains smooth navigation
/// context instead of discrete page jumps. Items are rendered using delegates that
/// control appearance and behavior, while filtering uses fuzzy matching with
/// character-level highlighting for search results.
///
/// # Navigation
///
/// - **Up/Down**: Move cursor through items with smooth viewport scrolling
/// - **Page Up/Down**: Jump by pages while maintaining cursor visibility
/// - **Home/End**: Jump to first/last item
/// - **/** : Start filtering
/// - **Enter**: Accept filter (while filtering)
/// - **Escape**: Cancel filter (while filtering)
/// - **Ctrl+C**: Clear active filter
///
/// # Filtering
///
/// The list supports fuzzy filtering with real-time preview:
/// - Type "/" to start filtering
/// - Type characters to filter items in real-time
/// - Matched characters are highlighted in the results
/// - Press Enter to accept the filter or Escape to cancel
///
/// # Styling
///
/// Visual appearance is controlled through the `ListStyles` struct and item delegates.
/// The list adapts to light/dark terminal themes automatically and supports
/// customizable colors, borders, and typography.
///
/// # Examples
///
/// ```
/// use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
///
/// let items = vec![
///     DefaultItem::new("Task 1", "Complete documentation"),
///     DefaultItem::new("Task 2", "Review pull requests"),
/// ];
/// let delegate = DefaultDelegate::new();
/// let list = Model::new(items, delegate, 80, 24);
/// ```
///
/// ## With Custom Items
///
/// ```
/// use bubbletea_widgets::list::{Item, Model, DefaultDelegate};
/// use std::fmt::Display;
///
/// #[derive(Clone)]
/// struct CustomItem {
///     title: String,
///     priority: u8,
/// }
///
/// impl Display for CustomItem {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "[{}] {}", self.priority, self.title)
///     }
/// }
///
/// impl Item for CustomItem {
///     fn filter_value(&self) -> String {
///         format!("{} priority:{}", self.title, self.priority)
///     }
/// }
///
/// let items = vec![
///     CustomItem { title: "Fix bug".to_string(), priority: 1 },
///     CustomItem { title: "Add feature".to_string(), priority: 2 },
/// ];
/// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
/// ```
pub struct Model<I: Item> {
    pub(super) title: String,
    pub(super) items: Vec<I>,
    pub(super) delegate: Box<dyn ItemDelegate<I> + Send + Sync>,

    // Pagination
    pub(super) paginator: paginator::Model,
    pub(super) per_page: usize,

    // UI State
    pub(super) show_title: bool,
    #[allow(dead_code)]
    pub(super) spinner: spinner::Model,
    pub(super) show_spinner: bool,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) styles: ListStyles,

    // Status bar
    pub(super) show_status_bar: bool,
    #[allow(dead_code)]
    pub(super) status_message_lifetime: usize,
    pub(super) status_item_singular: Option<String>,
    pub(super) status_item_plural: Option<String>,

    // Pagination display
    pub(super) show_pagination: bool,

    // Help
    pub(super) help: help::Model,
    pub(super) show_help: bool,
    pub(super) keymap: ListKeyMap,

    // State
    pub(super) filter_state: FilterState,
    pub(super) filtered_items: Vec<FilteredItem<I>>,
    pub(super) cursor: usize,
    /// First visible item index for smooth scrolling.
    ///
    /// This field tracks the index of the first item visible in the current viewport.
    /// It enables smooth, context-preserving scrolling behavior instead of discrete
    /// page jumps. The viewport scrolls automatically when the cursor moves outside
    /// the visible area, maintaining visual continuity.
    pub(super) viewport_start: usize,

    // Filter
    pub(super) filter_input: textinput::Model,
}

impl<I: Item + Send + Sync + 'static> Model<I> {
    /// Creates a new list with the provided items, delegate, and dimensions.
    ///
    /// This is the primary constructor for creating a list component. The delegate
    /// controls how items are rendered and behave, while the dimensions determine
    /// the initial size for layout calculations.
    ///
    /// # Arguments
    ///
    /// * `items` - Vector of items to display in the list
    /// * `delegate` - Item delegate that controls rendering and behavior
    /// * `width` - Initial width in terminal columns (can be updated later)
    /// * `height` - Initial height in terminal rows (affects pagination)
    ///
    /// # Returns
    ///
    /// A new `Model<I>` configured with default settings:
    /// - Title set to "List"
    /// - 10 items per page
    /// - Cursor at position 0
    /// - All items initially visible (no filtering)
    /// - Status bar enabled with default item names
    ///
    /// # Examples
    ///
    /// ```
    /// use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    ///
    /// let items = vec![
    ///     DefaultItem::new("First", "Description 1"),
    ///     DefaultItem::new("Second", "Description 2"),
    /// ];
    ///
    /// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
    /// assert_eq!(list.len(), 2);
    /// ```
    pub fn new<D>(items: Vec<I>, delegate: D, width: usize, height: usize) -> Self
    where
        D: ItemDelegate<I> + Send + Sync + 'static,
    {
        let styles = ListStyles::default();
        let mut paginator = paginator::Model::new();
        let per_page = 10;
        paginator.set_per_page(per_page);
        paginator.set_total_items(items.len());

        // Set dots mode by default (like Go version) and apply styled dots
        paginator.paginator_type = paginator::Type::Dots;
        paginator.active_dot = styles.active_pagination_dot.render("");
        paginator.inactive_dot = styles.inactive_pagination_dot.render("");

        Self {
            title: "List".to_string(),
            items,
            delegate: Box::new(delegate),
            paginator,
            per_page,
            show_title: true,
            spinner: spinner::new(&[]),
            show_spinner: false,
            width,
            height,
            styles,
            show_status_bar: true,
            status_message_lifetime: 1,
            status_item_singular: None,
            status_item_plural: None,
            show_pagination: true,
            help: help::Model::new(),
            show_help: true,
            keymap: ListKeyMap::default(),
            filter_state: FilterState::Unfiltered,
            filtered_items: vec![],
            cursor: 0,
            viewport_start: 0,
            filter_input: textinput::new(),
        }
    }

    /// Sets the items displayed in the list.
    ///
    /// This method replaces all current items with the provided vector.
    /// The cursor is reset to position 0, and pagination is recalculated
    /// based on the new item count.
    ///
    /// # Arguments
    ///
    /// * `items` - Vector of new items to display
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    ///
    /// let items = vec![
    ///     DefaultItem::new("Apple", "Red fruit"),
    ///     DefaultItem::new("Banana", "Yellow fruit"),
    /// ];
    /// list.set_items(items);
    /// assert_eq!(list.len(), 2);
    /// ```
    pub fn set_items(&mut self, items: Vec<I>) {
        self.items = items;
        self.cursor = 0;
        self.update_pagination();
    }

    /// Returns a vector of currently visible items.
    ///
    /// The returned items reflect the current filtering state:
    /// - When unfiltered: returns all items
    /// - When filtered: returns only items matching the current filter
    ///
    /// # Returns
    ///
    /// A vector containing clones of all currently visible items.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let items = vec![
    ///     DefaultItem::new("First", "Description 1"),
    ///     DefaultItem::new("Second", "Description 2"),
    /// ];
    ///
    /// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
    /// let visible = list.visible_items();
    /// assert_eq!(visible.len(), 2);
    /// ```
    pub fn visible_items(&self) -> Vec<I> {
        if self.filter_state == FilterState::Unfiltered {
            self.items.clone()
        } else {
            self.filtered_items
                .iter()
                .map(|fi| fi.item.clone())
                .collect()
        }
    }

    /// Sets the filter text without applying the filter.
    ///
    /// This method updates the filter input text but does not trigger
    /// the filtering process. It's primarily used for programmatic
    /// filter setup or testing.
    ///
    /// # Arguments
    ///
    /// * `s` - The filter text to set
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.set_filter_text("search term");
    /// // Filter text is set but not applied until filtering is activated
    /// ```
    pub fn set_filter_text(&mut self, s: &str) {
        self.filter_input.set_value(s);
    }

    /// Sets the current filtering state.
    ///
    /// This method directly controls the list's filtering state without
    /// triggering filter application. It's useful for programmatic state
    /// management or testing specific filter conditions.
    ///
    /// # Arguments
    ///
    /// * `st` - The new filter state to set
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem, FilterState};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.set_filter_state(FilterState::Filtering);
    /// // List is now in filtering mode
    /// ```
    pub fn set_filter_state(&mut self, st: FilterState) {
        self.filter_state = st;
    }

    /// Sets custom singular and plural names for status bar items.
    ///
    /// The status bar displays item counts using these names. If not set,
    /// defaults to "item" and "items".
    ///
    /// # Arguments
    ///
    /// * `singular` - Name for single item (e.g., "task")
    /// * `plural` - Name for multiple items (e.g., "tasks")
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.set_status_bar_item_name("task", "tasks");
    /// // Status bar will now show "1 task" or "5 tasks"
    /// ```
    pub fn set_status_bar_item_name(&mut self, singular: &str, plural: &str) {
        self.status_item_singular = Some(singular.to_string());
        self.status_item_plural = Some(plural.to_string());
    }

    /// Updates the list dimensions and recalculates layout.
    ///
    /// This method allows dynamic resizing of the list to match terminal
    /// size changes, similar to the Go bubbles list's `SetSize` method.
    /// It updates both width and height, then recalculates pagination
    /// to show the appropriate number of items.
    ///
    /// # Arguments
    ///
    /// * `width` - New width in terminal columns
    /// * `height` - New height in terminal rows
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// 
    /// // Resize list to match new terminal size
    /// list.set_size(100, 30);
    /// assert_eq!(list.width(), 100);
    /// assert_eq!(list.height(), 30);
    /// ```
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.update_pagination(); // Recalculate items per page
    }

    /// Returns the current width of the list.
    ///
    /// # Returns
    ///
    /// The current width in terminal columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the current height of the list.
    ///
    /// # Returns
    ///
    /// The current height in terminal rows.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Updates pagination settings based on current item count and page size.
    ///
    /// This method recalculates pagination after changes to item count or
    /// page size. It's called automatically after operations that affect
    /// the visible item count.
    pub(super) fn update_pagination(&mut self) {
        let total = self.len();
        self.paginator.set_total_items(total);

        // Calculate how many items can fit in the available height
        if self.height > 0 {
            let item_height = self.delegate.height() + self.delegate.spacing();
            
            // Header now includes title AND status line (like Go version)
            let header_height = if self.show_title && self.show_status_bar { 2 } else { 1 };
            
            // Footer includes help (1 line) + optional pagination dots
            let footer_height = if self.show_help { 1 } else { 0 } + 
                               if self.show_pagination { 1 } else { 0 };

            let available_height = self.height.saturating_sub(header_height + footer_height);
            let items_per_page = if item_height > 0 {
                (available_height / item_height).max(1)
            } else {
                5 // Match Go version default
            };

            self.per_page = items_per_page;
            self.paginator.set_per_page(items_per_page);
        }
    }

    /// Returns the number of currently visible items.
    ///
    /// This count reflects the items actually visible to the user:
    /// - When unfiltered: returns the total number of items
    /// - When filtering is active: returns only the count of matching items
    ///
    /// # Returns
    ///
    /// The number of items currently visible in the list.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let items = vec![
    ///     DefaultItem::new("Apple", "Red"),
    ///     DefaultItem::new("Banana", "Yellow"),
    /// ];
    /// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
    /// assert_eq!(list.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        if self.filter_state == FilterState::Unfiltered {
            self.items.len()
        } else {
            self.filtered_items.len()
        }
    }

    /// Returns whether the list has no visible items.
    ///
    /// # Returns
    ///
    /// `true` if there are no currently visible items, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// assert!(list.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the currently selected item.
    ///
    /// The selected item is the one at the current cursor position. If the list
    /// is empty or the cursor is out of bounds, returns `None`.
    ///
    /// # Returns
    ///
    /// A reference to the selected item, or `None` if no valid selection exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let items = vec![
    ///     DefaultItem::new("First", "Description 1"),
    ///     DefaultItem::new("Second", "Description 2"),
    /// ];
    /// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
    ///
    /// if let Some(selected) = list.selected_item() {
    ///     println!("Selected: {}", selected);
    /// }
    /// ```
    pub fn selected_item(&self) -> Option<&I> {
        if self.filter_state == FilterState::Unfiltered {
            self.items.get(self.cursor)
        } else {
            self.filtered_items.get(self.cursor).map(|fi| &fi.item)
        }
    }

    /// Returns the current cursor position.
    ///
    /// The cursor represents the currently selected item index within the
    /// visible (possibly filtered) list. This is always relative to the
    /// currently visible items, not the original full list.
    ///
    /// # Returns
    ///
    /// The zero-based index of the currently selected item.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let items = vec![
    ///     DefaultItem::new("First", "Description"),
    ///     DefaultItem::new("Second", "Description"),
    /// ];
    /// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
    /// assert_eq!(list.cursor(), 0); // Initially at first item
    /// ```
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns fuzzy match character indices for a given original item index.
    ///
    /// This method finds the character positions that matched the current filter
    /// for a specific item identified by its original index in the full items list.
    /// These indices can be used for character-level highlighting in custom delegates.
    ///
    /// # Arguments
    ///
    /// * `original_index` - The original index of the item in the full items list
    ///
    /// # Returns
    ///
    /// A reference to the vector of character indices that matched the filter,
    /// or `None` if no matches exist for this item or if filtering is not active.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let items = vec![DefaultItem::new("Apple", "Red fruit")];
    /// let mut list = Model::new(items, DefaultDelegate::new(), 80, 24);
    ///
    /// // Apply a filter first
    /// list.set_filter_text("app");
    /// // In a real application, this would be done through user interaction
    ///
    /// if let Some(matches) = list.matches_for_original_item(0) {
    ///     // matches contains the character indices that matched "app" in "Apple"
    ///     println!("Matched characters at indices: {:?}", matches);
    /// }
    /// ```
    pub fn matches_for_original_item(&self, original_index: usize) -> Option<&Vec<usize>> {
        self.filtered_items
            .iter()
            .find(|fi| fi.index == original_index)
            .map(|fi| &fi.matches)
    }

    // === Builder Pattern Methods ===

    /// Sets the list title (builder pattern).
    ///
    /// The title is displayed at the top of the list when not filtering.
    /// During filtering, the title is replaced with the filter input interface.
    ///
    /// # Arguments
    ///
    /// * `title` - The new title for the list
    ///
    /// # Returns
    ///
    /// Self, for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24)
    ///     .with_title("My Tasks");
    /// ```
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    /// Sets pagination display visibility (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show pagination, `false` to hide it
    ///
    /// # Returns
    ///
    /// Self, for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24)
    ///     .with_show_pagination(false);
    /// assert!(!list.show_pagination());
    /// ```
    pub fn with_show_pagination(mut self, show: bool) -> Self {
        self.show_pagination = show;
        self
    }

    /// Sets the pagination type (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `pagination_type` - The type of pagination to display
    ///
    /// # Returns
    ///
    /// Self, for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// # use bubbletea_widgets::paginator::Type;
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24)
    ///     .with_pagination_type(Type::Dots);
    /// assert_eq!(list.pagination_type(), Type::Dots);
    /// ```
    pub fn with_pagination_type(mut self, pagination_type: paginator::Type) -> Self {
        self.paginator.paginator_type = pagination_type;
        self
    }

    /// Sets title display visibility (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show title, `false` to hide it
    ///
    /// # Returns
    ///
    /// Self, for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24)
    ///     .with_show_title(false);
    /// assert!(!list.show_title());
    /// ```
    pub fn with_show_title(mut self, show: bool) -> Self {
        self.show_title = show;
        self
    }

    /// Sets status bar display visibility (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show status bar, `false` to hide it
    ///
    /// # Returns
    ///
    /// Self, for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24)
    ///     .with_show_status_bar(false);
    /// assert!(!list.show_status_bar());
    /// ```
    pub fn with_show_status_bar(mut self, show: bool) -> Self {
        self.show_status_bar = show;
        self
    }

    /// Sets spinner display visibility (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show spinner, `false` to hide it
    ///
    /// # Returns
    ///
    /// Self, for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24)
    ///     .with_show_spinner(true);
    /// assert!(list.show_spinner());
    /// ```
    pub fn with_show_spinner(mut self, show: bool) -> Self {
        self.show_spinner = show;
        self
    }

    /// Sets help display visibility (builder pattern).
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show help, `false` to hide it
    ///
    /// # Returns
    ///
    /// Self, for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24)
    ///     .with_show_help(true);
    /// assert!(list.show_help());
    /// ```
    pub fn with_show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    /// Sets the list's styling configuration (builder pattern).
    ///
    /// This replaces all current styles with the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `styles` - The styling configuration to apply
    ///
    /// # Returns
    ///
    /// Self, for method chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// # use bubbletea_widgets::list::style::ListStyles;
    /// let custom_styles = ListStyles::default();
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24)
    ///     .with_styles(custom_styles);
    /// ```
    pub fn with_styles(mut self, styles: ListStyles) -> Self {
        self.styles = styles;
        self
    }

    // === UI Component Toggles and Access ===

    /// Returns whether pagination is currently shown.
    ///
    /// # Returns
    ///
    /// `true` if pagination is displayed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// assert!(list.show_pagination()); // pagination is shown by default
    /// ```
    pub fn show_pagination(&self) -> bool {
        self.show_pagination
    }

    /// Sets whether pagination should be displayed.
    ///
    /// When disabled, the pagination section will not be rendered in the list view,
    /// but pagination state and navigation will continue to work normally.
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show pagination, `false` to hide it
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.set_show_pagination(false);
    /// assert!(!list.show_pagination());
    /// ```
    pub fn set_show_pagination(&mut self, show: bool) {
        self.show_pagination = show;
    }

    /// Toggles pagination display on/off.
    ///
    /// This is a convenience method that flips the current pagination display state.
    ///
    /// # Returns
    ///
    /// The new pagination display state after toggling.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let new_state = list.toggle_pagination();
    /// assert_eq!(new_state, list.show_pagination());
    /// ```
    pub fn toggle_pagination(&mut self) -> bool {
        self.show_pagination = !self.show_pagination;
        self.show_pagination
    }

    /// Returns the current pagination type (Arabic or Dots).
    ///
    /// # Returns
    ///
    /// The pagination type currently configured for this list.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// # use bubbletea_widgets::paginator::Type;
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let pagination_type = list.pagination_type();
    /// ```
    pub fn pagination_type(&self) -> paginator::Type {
        self.paginator.paginator_type
    }

    /// Sets the pagination display type.
    ///
    /// This controls how pagination is rendered:
    /// - `paginator::Type::Arabic`: Shows "1/5" style numbering
    /// - `paginator::Type::Dots`: Shows "• ○ • ○ •" style dots
    ///
    /// # Arguments
    ///
    /// * `pagination_type` - The type of pagination to display
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// # use bubbletea_widgets::paginator::Type;
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.set_pagination_type(Type::Dots);
    /// assert_eq!(list.pagination_type(), Type::Dots);
    /// ```
    pub fn set_pagination_type(&mut self, pagination_type: paginator::Type) {
        self.paginator.paginator_type = pagination_type;
    }

    // === Item Manipulation Methods ===

    /// Inserts an item at the specified index.
    ///
    /// All items at and after the specified index are shifted to the right.
    /// The cursor and pagination are updated appropriately.
    ///
    /// # Arguments
    ///
    /// * `index` - The position to insert the item at
    /// * `item` - The item to insert
    ///
    /// # Panics
    ///
    /// Panics if `index > len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.insert_item(0, DefaultItem::new("First", "Description"));
    /// assert_eq!(list.len(), 1);
    /// ```
    pub fn insert_item(&mut self, index: usize, item: I) {
        self.items.insert(index, item);
        // Clear any active filter since item indices have changed
        if self.filter_state != FilterState::Unfiltered {
            self.filter_state = FilterState::Unfiltered;
            self.filtered_items.clear();
        }
        // Update cursor if needed to maintain current selection
        if index <= self.cursor {
            self.cursor = self
                .cursor
                .saturating_add(1)
                .min(self.items.len().saturating_sub(1));
        }
        self.update_pagination();
    }

    /// Removes and returns the item at the specified index.
    ///
    /// All items after the specified index are shifted to the left.
    /// The cursor and pagination are updated appropriately.
    ///
    /// # Arguments
    ///
    /// * `index` - The position to remove the item from
    ///
    /// # Returns
    ///
    /// The removed item.
    ///
    /// # Panics
    ///
    /// Panics if `index >= len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list = Model::new(
    ///     vec![DefaultItem::new("First", "Desc")],
    ///     DefaultDelegate::new(), 80, 24
    /// );
    /// let removed = list.remove_item(0);
    /// assert_eq!(list.len(), 0);
    /// ```
    pub fn remove_item(&mut self, index: usize) -> I {
        if index >= self.items.len() {
            panic!("Index out of bounds");
        }

        // Check if the item can be removed
        if !self.delegate.can_remove(index, &self.items[index]) {
            panic!("Item cannot be removed");
        }

        // Call the on_remove callback before removal
        let item_ref = &self.items[index];
        let _ = self.delegate.on_remove(index, item_ref);

        let item = self.items.remove(index);
        // Clear any active filter since item indices have changed
        if self.filter_state != FilterState::Unfiltered {
            self.filter_state = FilterState::Unfiltered;
            self.filtered_items.clear();
        }
        // Update cursor to maintain valid position
        if !self.items.is_empty() {
            if index < self.cursor {
                self.cursor = self.cursor.saturating_sub(1);
            } else if self.cursor >= self.items.len() {
                self.cursor = self.items.len().saturating_sub(1);
            }
        } else {
            self.cursor = 0;
        }
        self.update_pagination();
        item
    }

    /// Moves an item from one position to another.
    ///
    /// The item at `from_index` is removed and inserted at `to_index`.
    /// The cursor is updated to follow the moved item if it was selected.
    ///
    /// # Arguments
    ///
    /// * `from_index` - The current position of the item to move
    /// * `to_index` - The target position to move the item to
    ///
    /// # Panics
    ///
    /// Panics if either index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list = Model::new(
    ///     vec![
    ///         DefaultItem::new("First", "1"),
    ///         DefaultItem::new("Second", "2"),
    ///     ],
    ///     DefaultDelegate::new(), 80, 24
    /// );
    /// list.move_item(0, 1); // Move "First" to position 1
    /// ```
    pub fn move_item(&mut self, from_index: usize, to_index: usize) {
        if from_index >= self.items.len() || to_index >= self.items.len() {
            panic!("Index out of bounds");
        }
        if from_index == to_index {
            return; // No movement needed
        }

        let item = self.items.remove(from_index);
        self.items.insert(to_index, item);

        // Clear any active filter since item indices have changed
        if self.filter_state != FilterState::Unfiltered {
            self.filter_state = FilterState::Unfiltered;
            self.filtered_items.clear();
        }

        // Update cursor to follow the moved item if it was selected
        if self.cursor == from_index {
            self.cursor = to_index;
        } else if from_index < self.cursor && to_index >= self.cursor {
            self.cursor = self.cursor.saturating_sub(1);
        } else if from_index > self.cursor && to_index <= self.cursor {
            self.cursor = self.cursor.saturating_add(1);
        }

        self.update_pagination();
    }

    /// Adds an item to the end of the list.
    ///
    /// This is equivalent to `insert_item(len(), item)`.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to add
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.push_item(DefaultItem::new("New Item", "Description"));
    /// assert_eq!(list.len(), 1);
    /// ```
    pub fn push_item(&mut self, item: I) {
        self.items.push(item);
        // Clear any active filter since item indices have changed
        if self.filter_state != FilterState::Unfiltered {
            self.filter_state = FilterState::Unfiltered;
            self.filtered_items.clear();
        }
        self.update_pagination();
    }

    /// Removes and returns the last item from the list.
    ///
    /// # Returns
    ///
    /// The last item, or `None` if the list is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list = Model::new(
    ///     vec![DefaultItem::new("Item", "Desc")],
    ///     DefaultDelegate::new(), 80, 24
    /// );
    /// let popped = list.pop_item();
    /// assert!(popped.is_some());
    /// assert_eq!(list.len(), 0);
    /// ```
    pub fn pop_item(&mut self) -> Option<I> {
        if self.items.is_empty() {
            return None;
        }

        let item = self.items.pop();
        // Clear any active filter since item indices have changed
        if self.filter_state != FilterState::Unfiltered {
            self.filter_state = FilterState::Unfiltered;
            self.filtered_items.clear();
        }
        // Update cursor if it's now out of bounds
        if self.cursor >= self.items.len() && !self.items.is_empty() {
            self.cursor = self.items.len() - 1;
        } else if self.items.is_empty() {
            self.cursor = 0;
        }
        self.update_pagination();
        item
    }

    /// Returns a reference to the underlying items collection.
    ///
    /// This provides read-only access to all items in the list,
    /// regardless of the current filtering state.
    ///
    /// # Returns
    ///
    /// A slice containing all items in the list.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let items = vec![DefaultItem::new("First", "1"), DefaultItem::new("Second", "2")];
    /// let list = Model::new(items.clone(), DefaultDelegate::new(), 80, 24);
    /// assert_eq!(list.items().len(), 2);
    /// assert_eq!(list.items()[0].to_string(), items[0].to_string());
    /// ```
    pub fn items(&self) -> &[I] {
        &self.items
    }

    /// Returns a mutable reference to the underlying items collection.
    ///
    /// This provides direct mutable access to the items. Note that after
    /// modifying items through this method, you should call `update_pagination()`
    /// to ensure pagination state remains consistent.
    ///
    /// **Warning**: Direct modification may invalidate the current filter state.
    /// Consider using the specific item manipulation methods instead.
    ///
    /// # Returns
    ///
    /// A mutable slice containing all items in the list.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list = Model::new(
    ///     vec![DefaultItem::new("First", "1")],
    ///     DefaultDelegate::new(), 80, 24
    /// );
    /// list.items_mut()[0] = DefaultItem::new("Modified", "Updated");
    /// assert_eq!(list.items()[0].to_string(), "Modified");
    /// ```
    pub fn items_mut(&mut self) -> &mut Vec<I> {
        // Clear filter state since items may be modified
        if self.filter_state != FilterState::Unfiltered {
            self.filter_state = FilterState::Unfiltered;
            self.filtered_items.clear();
        }
        &mut self.items
    }

    /// Returns the total number of items in the list.
    ///
    /// This returns the count of all items, not just visible items.
    /// For visible items count, use `len()`.
    ///
    /// # Returns
    ///
    /// The total number of items in the underlying collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list = Model::new(
    ///     vec![DefaultItem::new("1", ""), DefaultItem::new("2", "")],
    ///     DefaultDelegate::new(), 80, 24
    /// );
    /// assert_eq!(list.items_len(), 2);
    /// ```
    pub fn items_len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the underlying items collection is empty.
    ///
    /// This checks the total items count, not just visible items.
    /// For visible items check, use `is_empty()`.
    ///
    /// # Returns
    ///
    /// `true` if there are no items in the underlying collection, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// assert!(list.items_empty());
    /// ```
    pub fn items_empty(&self) -> bool {
        self.items.is_empty()
    }

    // === UI Component Access and Styling ===

    /// Returns whether the title is currently shown.
    ///
    /// # Returns
    ///
    /// `true` if the title is displayed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// assert!(list.show_title()); // title is shown by default
    /// ```
    pub fn show_title(&self) -> bool {
        self.show_title
    }

    /// Sets whether the title should be displayed.
    ///
    /// When disabled, the title section will not be rendered in the list view.
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show the title, `false` to hide it
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.set_show_title(false);
    /// assert!(!list.show_title());
    /// ```
    pub fn set_show_title(&mut self, show: bool) {
        self.show_title = show;
    }

    /// Toggles title display on/off.
    ///
    /// This is a convenience method that flips the current title display state.
    ///
    /// # Returns
    ///
    /// The new title display state after toggling.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let new_state = list.toggle_title();
    /// assert_eq!(new_state, list.show_title());
    /// ```
    pub fn toggle_title(&mut self) -> bool {
        self.show_title = !self.show_title;
        self.show_title
    }

    /// Returns whether the status bar is currently shown.
    ///
    /// # Returns
    ///
    /// `true` if the status bar is displayed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// assert!(list.show_status_bar()); // status bar is shown by default
    /// ```
    pub fn show_status_bar(&self) -> bool {
        self.show_status_bar
    }

    /// Sets whether the status bar should be displayed.
    ///
    /// When disabled, the status bar section will not be rendered in the list view.
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show the status bar, `false` to hide it
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.set_show_status_bar(false);
    /// assert!(!list.show_status_bar());
    /// ```
    pub fn set_show_status_bar(&mut self, show: bool) {
        self.show_status_bar = show;
    }

    /// Toggles status bar display on/off.
    ///
    /// This is a convenience method that flips the current status bar display state.
    ///
    /// # Returns
    ///
    /// The new status bar display state after toggling.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let new_state = list.toggle_status_bar();
    /// assert_eq!(new_state, list.show_status_bar());
    /// ```
    pub fn toggle_status_bar(&mut self) -> bool {
        self.show_status_bar = !self.show_status_bar;
        self.show_status_bar
    }

    /// Returns whether the spinner is currently shown.
    ///
    /// # Returns
    ///
    /// `true` if the spinner is displayed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// assert!(!list.show_spinner()); // spinner is hidden by default
    /// ```
    pub fn show_spinner(&self) -> bool {
        self.show_spinner
    }

    /// Sets whether the spinner should be displayed.
    ///
    /// When enabled, the spinner will be rendered as part of the list view,
    /// typically to indicate loading state.
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show the spinner, `false` to hide it
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.set_show_spinner(true);
    /// assert!(list.show_spinner());
    /// ```
    pub fn set_show_spinner(&mut self, show: bool) {
        self.show_spinner = show;
    }

    /// Toggles spinner display on/off.
    ///
    /// This is a convenience method that flips the current spinner display state.
    ///
    /// # Returns
    ///
    /// The new spinner display state after toggling.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let new_state = list.toggle_spinner();
    /// assert_eq!(new_state, list.show_spinner());
    /// ```
    pub fn toggle_spinner(&mut self) -> bool {
        self.show_spinner = !self.show_spinner;
        self.show_spinner
    }

    /// Returns a reference to the spinner model.
    ///
    /// This allows access to the underlying spinner for customization
    /// and state management.
    ///
    /// # Returns
    ///
    /// A reference to the spinner model.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let spinner = list.spinner();
    /// ```
    pub fn spinner(&self) -> &spinner::Model {
        &self.spinner
    }

    /// Returns a mutable reference to the spinner model.
    ///
    /// This allows modification of the underlying spinner for customization
    /// and state management.
    ///
    /// # Returns
    ///
    /// A mutable reference to the spinner model.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let spinner = list.spinner_mut();
    /// ```
    pub fn spinner_mut(&mut self) -> &mut spinner::Model {
        &mut self.spinner
    }

    /// Returns whether the help is currently shown.
    ///
    /// # Returns
    ///
    /// `true` if help is displayed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// assert!(!list.show_help()); // help is hidden by default
    /// ```
    pub fn show_help(&self) -> bool {
        self.show_help
    }

    /// Sets whether help should be displayed.
    ///
    /// When enabled, help text will be rendered as part of the list view,
    /// showing available key bindings and controls.
    ///
    /// # Arguments
    ///
    /// * `show` - `true` to show help, `false` to hide it
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list.set_show_help(true);
    /// assert!(list.show_help());
    /// ```
    pub fn set_show_help(&mut self, show: bool) {
        self.show_help = show;
    }

    /// Toggles help display on/off.
    ///
    /// This is a convenience method that flips the current help display state.
    ///
    /// # Returns
    ///
    /// The new help display state after toggling.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let new_state = list.toggle_help();
    /// assert_eq!(new_state, list.show_help());
    /// ```
    pub fn toggle_help(&mut self) -> bool {
        self.show_help = !self.show_help;
        self.show_help
    }

    /// Returns a reference to the help model.
    ///
    /// This allows access to the underlying help system for customization
    /// and state management.
    ///
    /// # Returns
    ///
    /// A reference to the help model.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let help = list.help();
    /// ```
    pub fn help(&self) -> &help::Model {
        &self.help
    }

    /// Returns a mutable reference to the help model.
    ///
    /// This allows modification of the underlying help system for customization
    /// and state management.
    ///
    /// # Returns
    ///
    /// A mutable reference to the help model.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let help = list.help_mut();
    /// ```
    pub fn help_mut(&mut self) -> &mut help::Model {
        &mut self.help
    }

    /// Returns a reference to the list's styling configuration.
    ///
    /// This provides read-only access to all visual styles used by the list,
    /// including title, item, status bar, pagination, and help styles.
    ///
    /// # Returns
    ///
    /// A reference to the `ListStyles` configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let styles = list.styles();
    /// // Access specific styles, e.g., styles.title, styles.pagination_style
    /// ```
    pub fn styles(&self) -> &ListStyles {
        &self.styles
    }

    /// Returns a mutable reference to the list's styling configuration.
    ///
    /// This provides direct mutable access to all visual styles used by the list.
    /// Changes to styles take effect immediately on the next render.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `ListStyles` configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// # use lipgloss_extras::prelude::*;
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let styles = list.styles_mut();
    /// styles.title = Style::new().foreground("#FF0000"); // Red title
    /// ```
    pub fn styles_mut(&mut self) -> &mut ListStyles {
        &mut self.styles
    }

    /// Sets the list's styling configuration.
    ///
    /// This replaces all current styles with the provided configuration.
    /// Changes take effect immediately on the next render.
    ///
    /// # Arguments
    ///
    /// * `styles` - The new styling configuration to apply
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// # use bubbletea_widgets::list::style::ListStyles;
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// let custom_styles = ListStyles::default();
    /// list.set_styles(custom_styles);
    /// ```
    pub fn set_styles(&mut self, styles: ListStyles) {
        // Update paginator dots from styled strings
        self.paginator.active_dot = styles.active_pagination_dot.render("");
        self.paginator.inactive_dot = styles.inactive_pagination_dot.render("");
        self.styles = styles;
    }

    /// Renders the status bar as a formatted string.
    ///
    /// The status bar shows the current selection position and total item count,
    /// using the custom item names if set. The format is "X/Y items" where X is
    /// the current cursor position + 1, and Y is the total item count.
    ///
    /// # Returns
    ///
    /// A formatted status string, or empty string if status bar is disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let items = vec![
    ///     DefaultItem::new("First", ""),
    ///     DefaultItem::new("Second", ""),
    /// ];
    /// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
    /// let status = list.status_view();
    /// assert!(status.contains("1/2"));
    /// ```
    pub fn status_view(&self) -> String {
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

    // === Advanced Filtering API ===
}
