//! Core types and traits for list components.
//!
//! This module contains the fundamental types and traits that define the interface
//! and behavior of list components. It includes:
//! - Item trait for displayable/filterable items
//! - ItemDelegate trait for custom rendering
//! - FilterState and FilterStateInfo for filter management
//! - Internal types for filtered item representation

use bubbletea_rs::{Cmd, Msg};
use std::fmt::Display;

/// Trait for items that can be displayed and filtered in a list.
///
/// Items must be displayable and cloneable to work with the list component.
/// The `filter_value()` method determines what text is used when searching
/// through items with the fuzzy filter.
///
/// # Examples
///
/// ```
/// use bubbletea_widgets::list::Item;
/// use std::fmt::Display;
///
/// #[derive(Clone)]
/// struct Task {
///     name: String,
///     description: String,
/// }
///
/// impl Display for Task {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "{}", self.name)
///     }
/// }
///
/// impl Item for Task {
///     fn filter_value(&self) -> String {
///         // Filter searches both name and description
///         format!("{} {}", self.name, self.description)
///     }
/// }
/// ```
pub trait Item: Display + Clone {
    /// Returns the text used for fuzzy filtering this item.
    ///
    /// This method should return a string that represents all searchable
    /// content for the item. The fuzzy matcher will search within this
    /// text when the user types a filter query.
    ///
    /// # Returns
    ///
    /// A string containing all text that should be searchable for this item.
    /// Common patterns include returning just the display name, or combining
    /// multiple fields like "name description tags".
    ///
    /// # Examples
    ///
    /// ```
    /// use bubbletea_widgets::list::Item;
    ///
    /// #[derive(Clone)]
    /// struct MyItem(String);
    ///
    /// impl std::fmt::Display for MyItem {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "{}", self.0)
    ///     }
    /// }
    ///
    /// impl Item for MyItem {
    ///     fn filter_value(&self) -> String {
    ///         self.0.clone()
    ///     }
    /// }
    /// ```
    fn filter_value(&self) -> String;
}

/// Trait for customizing how list items are rendered and behave.
///
/// The ItemDelegate trait allows you to completely customize the appearance
/// and behavior of items in a list. This includes rendering individual items,
/// defining their height and spacing, and handling custom update logic.
///
/// # Examples
///
/// ```
/// # use bubbletea_widgets::list::{Item, ItemDelegate};
/// # use bubbletea_rs::{Cmd, Msg};
/// // Multi-line delegate that shows item + description
/// struct DetailedDelegate;
///
/// impl<I: Item> ItemDelegate<I> for DetailedDelegate {
/// #   fn render(&self, _m: &bubbletea_widgets::list::Model<I>, _index: usize, item: &I) -> String { item.to_string() }
///     fn height(&self) -> usize {
///         2 // Title line + description line
///     }
/// #   fn spacing(&self) -> usize { 0 }
/// #   fn update(&self, _msg: &Msg, _m: &mut bubbletea_widgets::list::Model<I>) -> Option<Cmd> { None }
/// }
/// ```
pub trait ItemDelegate<I: Item> {
    /// Renders an item as a string for display in the list.
    ///
    /// This method is called for each visible item and should return a
    /// styled string representation. The method receives the complete
    /// model state, the item's original index, and the item itself.
    ///
    /// # Arguments
    ///
    /// * `m` - The complete list model with current state
    /// * `index` - The original index of this item in the full items list
    /// * `item` - The item to render
    ///
    /// # Returns
    ///
    /// A formatted string with any necessary ANSI styling codes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Item, ItemDelegate, Model};
    /// # use bubbletea_rs::{Cmd, Msg};
    /// struct CustomDelegate;
    ///
    /// impl<I: Item + Send + Sync + 'static> ItemDelegate<I> for CustomDelegate {
    ///     fn render(&self, m: &Model<I>, index: usize, item: &I) -> String {
    ///         if index == m.cursor() {
    ///             format!("> {}", item)  // Highlight selected item
    ///         } else {
    ///             format!("  {}", item)  // Normal item
    ///         }
    ///     }
    /// # fn height(&self) -> usize { 1 }
    /// # fn spacing(&self) -> usize { 0 }
    /// # fn update(&self, _msg: &Msg, _m: &mut Model<I>) -> Option<Cmd> { None }
    /// }
    /// ```
    fn render(&self, m: &super::Model<I>, index: usize, item: &I) -> String;

    /// Returns the height in terminal lines that each item occupies.
    ///
    /// This value is used by the list for layout calculations, viewport
    /// sizing, and scroll positioning. It should include any line breaks
    /// or multi-line content in your rendered items.
    ///
    /// # Returns
    ///
    /// The number of terminal lines each item will occupy when rendered.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Item, ItemDelegate};
    /// # use bubbletea_rs::{Cmd, Msg};
    /// struct MultiLineDelegate;
    ///
    /// impl<I: Item> ItemDelegate<I> for MultiLineDelegate {
    ///     fn height(&self) -> usize {
    ///         3  // Item title + description + blank line
    ///     }
    /// # fn render(&self, _m: &bubbletea_widgets::list::Model<I>, _index: usize, item: &I) -> String { item.to_string() }
    /// # fn spacing(&self) -> usize { 0 }
    /// # fn update(&self, _msg: &Msg, _m: &mut bubbletea_widgets::list::Model<I>) -> Option<Cmd> { None }
    /// }
    /// ```
    fn height(&self) -> usize;

    /// Returns the number of blank lines to insert between items.
    ///
    /// This spacing is added between each item in the list to improve
    /// readability and visual separation. The spacing is in addition to
    /// the height returned by `height()`.
    ///
    /// # Returns
    ///
    /// The number of blank lines to insert between rendered items.
    fn spacing(&self) -> usize;

    /// Handles update messages for the delegate.
    ///
    /// This method allows delegates to respond to messages and potentially
    /// modify the list state or return commands. Most delegates don't need
    /// custom update logic and can return `None`.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to handle
    /// * `m` - Mutable reference to the list model
    ///
    /// # Returns
    ///
    /// An optional command to be executed by the runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Item, ItemDelegate, Model};
    /// # use bubbletea_rs::{Cmd, Msg, KeyMsg};
    /// # use crossterm::event::KeyCode;
    /// struct InteractiveDelegate;
    ///
    /// impl<I: Item> ItemDelegate<I> for InteractiveDelegate {
    ///     fn update(&self, msg: &Msg, m: &mut Model<I>) -> Option<Cmd> {
    ///         // Example: Handle custom key press
    ///         if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
    ///             if key_msg.key == KeyCode::Char(' ') {
    ///                 // Custom space key behavior
    ///                 return None;
    ///             }
    ///         }
    ///         None
    ///     }
    /// # fn render(&self, _m: &Model<I>, _index: usize, item: &I) -> String { item.to_string() }
    /// # fn height(&self) -> usize { 1 }
    /// # fn spacing(&self) -> usize { 0 }
    /// }
    /// ```
    fn update(&self, msg: &Msg, m: &mut super::Model<I>) -> Option<Cmd>;

    /// Returns key bindings for the short help view.
    ///
    /// This method provides a compact set of key bindings that will be
    /// displayed in short help views. The bindings should represent the
    /// most important or commonly used actions for this delegate.
    ///
    /// # Returns
    ///
    /// A vector of key bindings for compact help display.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Item, ItemDelegate, Model};
    /// # use bubbletea_widgets::key::{self, Binding};
    /// # use bubbletea_rs::{Cmd, Msg};
    /// struct HelpfulDelegate {
    ///     select_key: key::Binding,
    /// }
    ///
    /// impl<I: Item> ItemDelegate<I> for HelpfulDelegate {
    ///     fn short_help(&self) -> Vec<key::Binding> {
    ///         vec![self.select_key.clone()]
    ///     }
    ///     
    /// # fn render(&self, _m: &Model<I>, _index: usize, item: &I) -> String { item.to_string() }
    /// # fn height(&self) -> usize { 1 }
    /// # fn spacing(&self) -> usize { 0 }
    /// # fn update(&self, _msg: &Msg, _m: &mut Model<I>) -> Option<Cmd> { None }
    /// }
    /// ```
    fn short_help(&self) -> Vec<crate::key::Binding> {
        vec![]
    }

    /// Returns key bindings for the full help view.
    ///
    /// This method organizes all key bindings into columns for display in
    /// expanded help views. Each inner vector represents a column of related
    /// key bindings.
    ///
    /// # Returns
    ///
    /// A vector of vectors, where each inner vector represents a column
    /// of related key bindings.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Item, ItemDelegate, Model};
    /// # use bubbletea_widgets::key::{self, Binding};
    /// # use bubbletea_rs::{Cmd, Msg};
    /// struct OrganizedDelegate {
    ///     navigation_keys: Vec<key::Binding>,
    ///     action_keys: Vec<key::Binding>,
    /// }
    ///
    /// impl<I: Item> ItemDelegate<I> for OrganizedDelegate {
    ///     fn full_help(&self) -> Vec<Vec<key::Binding>> {
    ///         vec![
    ///             self.navigation_keys.clone(),
    ///             self.action_keys.clone(),
    ///         ]
    ///     }
    ///     
    /// # fn render(&self, _m: &Model<I>, _index: usize, item: &I) -> String { item.to_string() }
    /// # fn height(&self) -> usize { 1 }
    /// # fn spacing(&self) -> usize { 0 }
    /// # fn update(&self, _msg: &Msg, _m: &mut Model<I>) -> Option<Cmd> { None }
    /// }
    /// ```
    fn full_help(&self) -> Vec<Vec<crate::key::Binding>> {
        vec![]
    }

    /// Called when an item is selected (e.g., Enter key pressed).
    ///
    /// This method is invoked when the user selects an item, typically by
    /// pressing Enter. It allows the delegate to perform custom actions
    /// or return commands in response to item selection.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the selected item in the original items list
    /// * `item` - A reference to the selected item
    ///
    /// # Returns
    ///
    /// An optional command to execute in response to the selection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Item, ItemDelegate, Model};
    /// # use bubbletea_rs::{Cmd, Msg};
    /// struct SelectableDelegate;
    ///
    /// impl<I: Item> ItemDelegate<I> for SelectableDelegate {
    ///     fn on_select(&self, index: usize, item: &I) -> Option<Cmd> {
    ///         // Log the selection and return a custom command
    ///         println!("Selected item {} at index {}", item, index);
    ///         None // Or return Some(your_command)
    ///     }
    ///     
    /// # fn render(&self, _m: &Model<I>, _index: usize, item: &I) -> String { item.to_string() }
    /// # fn height(&self) -> usize { 1 }
    /// # fn spacing(&self) -> usize { 0 }
    /// # fn update(&self, _msg: &Msg, _m: &mut Model<I>) -> Option<Cmd> { None }
    /// }
    /// ```
    fn on_select(&self, _index: usize, _item: &I) -> Option<Cmd> {
        None
    }

    /// Called when an item is about to be removed.
    ///
    /// This method is invoked before an item is removed from the list,
    /// allowing the delegate to perform cleanup actions or return commands.
    /// Note that this is called by the list's removal methods, not automatically
    /// by user actions.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the item being removed
    /// * `item` - A reference to the item being removed
    ///
    /// # Returns
    ///
    /// An optional command to execute in response to the removal.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Item, ItemDelegate, Model};
    /// # use bubbletea_rs::{Cmd, Msg};
    /// struct TrackingDelegate;
    ///
    /// impl<I: Item> ItemDelegate<I> for TrackingDelegate {
    ///     fn on_remove(&self, index: usize, item: &I) -> Option<Cmd> {
    ///         // Log the removal
    ///         println!("Removing item {} at index {}", item, index);
    ///         None
    ///     }
    ///     
    /// # fn render(&self, _m: &Model<I>, _index: usize, item: &I) -> String { item.to_string() }
    /// # fn height(&self) -> usize { 1 }
    /// # fn spacing(&self) -> usize { 0 }
    /// # fn update(&self, _msg: &Msg, _m: &mut Model<I>) -> Option<Cmd> { None }
    /// }
    /// ```
    fn on_remove(&self, _index: usize, _item: &I) -> Option<Cmd> {
        None
    }

    /// Determines whether an item can be removed.
    ///
    /// This method is called by the list to determine if an item at a given
    /// index is allowed to be removed. This can be used to implement
    /// protection for certain items or conditional removal logic.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the item being checked for removal
    /// * `item` - A reference to the item being checked
    ///
    /// # Returns
    ///
    /// `true` if the item can be removed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Item, ItemDelegate, Model};
    /// # use bubbletea_rs::{Cmd, Msg};
    /// struct ProtectedDelegate;
    ///
    /// impl<I: Item> ItemDelegate<I> for ProtectedDelegate {
    ///     fn can_remove(&self, index: usize, item: &I) -> bool {
    ///         // Don't allow removal of the first item
    ///         index != 0
    ///     }
    ///     
    /// # fn render(&self, _m: &Model<I>, _index: usize, item: &I) -> String { item.to_string() }
    /// # fn height(&self) -> usize { 1 }
    /// # fn spacing(&self) -> usize { 0 }
    /// # fn update(&self, _msg: &Msg, _m: &mut Model<I>) -> Option<Cmd> { None }
    /// }
    /// ```
    fn can_remove(&self, _index: usize, _item: &I) -> bool {
        true
    }
}

/// Internal representation of a filtered item with fuzzy match indices.
///
/// This structure stores both the original item index and the character indices
/// that matched the fuzzy search pattern. It's used internally by the filtering
/// system to maintain the relationship between filtered results and original items.
#[derive(Debug, Clone)]
pub(super) struct FilteredItem<I: Item> {
    /// Original index of this item in the full items list.
    pub index: usize,
    /// The actual item data.
    pub item: I,
    /// Character indices that matched the filter query (for highlighting).
    pub matches: Vec<usize>,
}

/// Represents the current filtering state of the list.
///
/// This enum tracks the three distinct phases of the filtering process:
/// - `Unfiltered`: No filtering active, all items visible
/// - `Filtering`: User actively typing, live filter preview
/// - `FilterApplied`: Filter accepted and applied, only matches visible
///
/// The transitions between states are:
/// ```text
/// Unfiltered → Filtering (user presses '/' to start filtering)
/// Filtering → FilterApplied (user presses Enter to accept filter)
/// Filtering → Unfiltered (user presses Esc to cancel, filter becomes empty)
/// FilterApplied → Filtering (user presses '/' to modify filter)
/// FilterApplied → Unfiltered (user clears filter)
/// ```
///
/// # Examples
///
/// ```
/// use bubbletea_widgets::list::FilterState;
///
/// let state = FilterState::Unfiltered;
/// assert_eq!(state, FilterState::Unfiltered);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterState {
    /// No filtering is active; all items are shown normally.
    ///
    /// In this state:
    /// - All items in the list are visible
    /// - The list title is shown in the header
    /// - No filter input box is displayed
    /// - Navigation keys work normally
    Unfiltered,

    /// User is actively typing a filter term; live filtering UI is shown.
    ///
    /// In this state:
    /// - A "Filter: > ___" input box replaces the header
    /// - Items are filtered in real-time as the user types
    /// - Only matching items are visible with character-level highlighting
    /// - Most navigation keys are disabled; typing adds to filter
    /// - Enter accepts the filter, Escape cancels it
    Filtering,

    /// A filter term has been applied; only matching items are shown.
    ///
    /// In this state:
    /// - Only items matching the filter are visible
    /// - The header shows "Title (filtered: N)" format
    /// - Navigation keys work normally on the filtered items
    /// - Character-level highlighting shows matched characters
    /// - Pressing '/' allows modifying the filter
    FilterApplied,
}

/// Detailed information about the current filter state.
///
/// This struct provides comprehensive information about the list's filtering state,
/// making it easy for applications to understand and react to filter conditions
/// without accessing internal list fields directly.
///
/// # Examples
///
/// ```
/// use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem, FilterStateInfo};
///
/// let items = vec![DefaultItem::new("Apple", "Red fruit")];
/// let mut list = Model::new(items, DefaultDelegate::new(), 80, 24);
///
/// let state_info = list.filter_state_info();
/// assert_eq!(state_info.match_count, 1);
/// assert_eq!(state_info.query, "");
/// assert!(!state_info.is_filtering);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterStateInfo {
    /// The current filter state.
    pub state: FilterState,
    /// The current filter query text.
    pub query: String,
    /// Number of items matching the current filter.
    pub match_count: usize,
    /// Whether any kind of filtering is currently active.
    pub is_filtering: bool,
    /// Whether the list is in the process of clearing the filter.
    pub is_clearing: bool,
}
