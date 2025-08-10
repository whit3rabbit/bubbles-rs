//! Main Model struct and core functionality for list components.
//!
//! This module contains the primary Model struct that represents a list component,
//! along with its basic construction, state management, and accessor methods.

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
    #[allow(dead_code)]
    pub(super) spinner: spinner::Model,
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) styles: ListStyles,

    // Status bar
    pub(super) show_status_bar: bool,
    #[allow(dead_code)]
    pub(super) status_message_lifetime: usize,
    pub(super) status_item_singular: Option<String>,
    pub(super) status_item_plural: Option<String>,

    // Help
    pub(super) help: help::Model,
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
        let mut paginator = paginator::Model::new();
        let per_page = 10;
        paginator.set_per_page(per_page);
        paginator.set_total_items(items.len());

        Self {
            title: "List".to_string(),
            items,
            delegate: Box::new(delegate),
            paginator,
            per_page,
            spinner: spinner::new(&[]),
            width,
            height,
            styles: ListStyles::default(),
            show_status_bar: true,
            status_message_lifetime: 1,
            status_item_singular: None,
            status_item_plural: None,
            help: help::Model::new(),
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

    /// Sets the list title.
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
    ///
    /// Or using the mutable method:
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// let mut list: Model<DefaultItem> = Model::new(vec![], DefaultDelegate::new(), 80, 24);
    /// list = list.with_title("My Tasks");
    /// ```
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
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
            let header_height = 1; // Title or filter input
            let footer_height = if self.show_status_bar { 2 } else { 0 }; // Status + help

            let available_height = self.height.saturating_sub(header_height + footer_height);
            let items_per_page = if item_height > 0 {
                (available_height / item_height).max(1)
            } else {
                10 // Fallback to default value
            };

            self.per_page = items_per_page;
            self.paginator.set_per_page(items_per_page);
        }
    }
}
