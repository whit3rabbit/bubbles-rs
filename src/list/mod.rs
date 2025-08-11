//! List component with filtering, pagination, contextual help, and customizable rendering.
//!
//! This module exposes a generic `Model<I: Item>` plus supporting traits and submodules:
//! - `Item`: Implement for your item type; must be `Display + Clone` and return a `filter_value()`
//! - `ItemDelegate`: Controls item `render`, `height`, `spacing`, and `update`
//! - Submodules: `defaultitem`, `keys`, and `style`
//!
//! ## Architecture Overview
//!
//! This list component uses several key architectural patterns for smooth interaction:
//!
//! ### 🎯 Core Design Principles
//! 1. **Viewport-Based Scrolling**: Maintains smooth, context-preserving navigation
//! 2. **Index Consistency**: Uses original item indices for cursor tracking across all states
//! 3. **Real-Time Filtering**: Integrates textinput component for responsive filter interaction
//! 4. **State-Driven UI**: Clear separation between filtering, navigation, and display states
//!
//! ### 🏗️ Key Components
//! - **Viewport Management**: `viewport_start` field tracks visible window position
//! - **Index Strategy**: Delegates receive original indices for consistent highlighting
//! - **Filter Integration**: Direct textinput forwarding preserves all input features
//! - **State Coordination**: Filtering states control UI behavior and key handling
//!
//! ### 📋 Implementation Strategy
//! - **Viewport Scrolling**: Only adjusts view when cursor moves outside visible bounds
//! - **Index Semantics**: Render delegates use original positions for cursor comparison
//! - **Filter States**: `Filtering` during typing, `FilterApplied` after acceptance
//! - **Event Handling**: KeyMsg forwarding maintains textinput component consistency
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

// Module declarations

/// Default item implementation and delegate for basic list functionality.
///
/// This module provides ready-to-use implementations for common list use cases:
/// - `DefaultItem`: A simple string-based item with title and description
/// - `DefaultDelegate`: A delegate that renders items with proper highlighting
/// - `DefaultItemStyles`: Customizable styling for default item rendering
///
/// These components handle fuzzy match highlighting, cursor styling, and basic
/// item representation without requiring custom implementations.
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
///
/// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
/// ```
pub mod defaultitem;

/// Key bindings and keyboard input handling for list navigation.
///
/// This module defines the key mapping system that controls how users interact
/// with the list component. It includes:
/// - `ListKeyMap`: Configurable key bindings for all list operations
/// - Default key mappings following common terminal UI conventions
/// - Support for custom key binding overrides
///
/// The key system integrates with the help system to provide contextual
/// keyboard shortcuts based on the current list state.
///
/// # Examples
///
/// ```
/// use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem, ListKeyMap};
///
/// let items = vec![DefaultItem::new("Item", "Description")];
/// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
///
/// // The key mappings are used internally by the list component
/// // They can be customized when creating custom list implementations
/// ```
pub mod keys;

/// Visual styling and theming for list components.
///
/// This module provides comprehensive styling options for customizing the
/// appearance of list components:
/// - `ListStyles`: Complete styling configuration for all visual elements
/// - Color schemes that adapt to light/dark terminal themes
/// - Typography and border styling options
/// - Default styles following terminal UI conventions
///
/// The styling system supports both built-in themes and complete customization
/// for applications with specific branding requirements.
///
/// # Examples
///
/// ```
/// use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem, ListStyles};
///
/// let items = vec![DefaultItem::new("Item", "Description")];
/// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
///
/// // Get the default styling - customization is done by modifying the struct fields
/// let default_styles = ListStyles::default();
/// // Styling can be customized by creating a new ListStyles instance
/// ```
pub mod style;

// Internal modules
mod api;
mod filtering;
mod model;
mod rendering;
mod types;

// Re-export public types from submodules

/// The main list component model.
///
/// `Model<I>` is a generic list component that can display any items implementing
/// the `Item` trait. It provides filtering, navigation, pagination, and customizable
/// rendering through the delegate pattern.
///
/// # Type Parameters
///
/// * `I` - The item type that must implement `Item + Send + Sync + 'static`
///
/// # Examples
///
/// ```
/// use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
///
/// let items = vec![
///     DefaultItem::new("Apple", "Red fruit"),
///     DefaultItem::new("Banana", "Yellow fruit"),
/// ];
/// let list = Model::new(items, DefaultDelegate::new(), 80, 24);
/// ```
pub use model::Model;

/// Key binding configuration for list navigation and interaction.
///
/// `ListKeyMap` defines all the keyboard shortcuts used for list operations
/// including navigation, filtering, and help. It can be customized to match
/// application-specific key binding preferences.
pub use keys::ListKeyMap;

/// Visual styling configuration for list appearance.
///
/// `ListStyles` contains all styling options for customizing the visual
/// appearance of list components including colors, typography, and borders.
/// It supports both light and dark terminal themes automatically.
pub use style::ListStyles;

/// Core traits and types for list functionality.
///
/// These are the fundamental building blocks for creating custom list items
/// and delegates:
///
/// - `Item`: Trait for displayable and filterable list items
/// - `ItemDelegate`: Trait for customizing item rendering and behavior  
/// - `FilterState`: Enum representing the current filtering state
/// - `FilterStateInfo`: Detailed information about filter status
///
/// # Examples
///
/// ```
/// use bubbletea_widgets::list::{Item, ItemDelegate, FilterState, Model};
/// use std::fmt::Display;
///
/// #[derive(Clone)]
/// struct MyItem {
///     name: String,
///     value: i32,
/// }
///
/// impl Display for MyItem {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "{}: {}", self.name, self.value)
///     }
/// }
///
/// impl Item for MyItem {
///     fn filter_value(&self) -> String {
///         format!("{} {}", self.name, self.value)
///     }
/// }
/// ```
pub use types::{FilterState, FilterStateInfo, Item, ItemDelegate};

/// Ready-to-use implementations for common list scenarios.
///
/// These provide drop-in functionality for typical list use cases:
///
/// - `DefaultItem`: Simple string-based items with title and description
/// - `DefaultDelegate`: Standard item rendering with highlighting support
/// - `DefaultItemStyles`: Styling configuration for default rendering
///
/// Perfect for prototyping or applications that don't need custom item types.
///
/// # Examples
///
/// ```
/// use bubbletea_widgets::list::{DefaultItem, DefaultDelegate, DefaultItemStyles, Model};
///
/// // Create items using the default implementation
/// let items = vec![
///     DefaultItem::new("First Item", "Description 1"),
///     DefaultItem::new("Second Item", "Description 2"),
/// ];
///
/// // Use the default delegate for rendering
/// let delegate = DefaultDelegate::new();
/// let list = Model::new(items, delegate, 80, 24);
/// ```
pub use defaultitem::{DefaultDelegate, DefaultItem, DefaultItemStyles};

use crate::{help, key};
use bubbletea_rs::{Cmd, KeyMsg, Model as BubbleTeaModel, Msg};
use crossterm::event::KeyCode;

// Help integration - provides contextual key bindings based on current state
impl<I: Item> help::KeyMap for Model<I> {
    /// Returns key bindings for compact help display.
    ///
    /// Provides a minimal set of the most important key bindings
    /// based on the current list state. The bindings change depending
    /// on whether the user is actively filtering or navigating.
    ///
    /// # Context-Sensitive Help
    ///
    /// - **While filtering**: Shows Enter (accept) and Escape (cancel) bindings
    /// - **Normal navigation**: Shows up/down navigation and filter activation
    fn short_help(&self) -> Vec<&key::Binding> {
        match self.filter_state {
            FilterState::Filtering => vec![&self.keymap.accept_filter, &self.keymap.cancel_filter],
            _ => vec![
                &self.keymap.cursor_up,
                &self.keymap.cursor_down,
                &self.keymap.filter,
                &self.keymap.quit,
                &self.keymap.show_full_help, // Add "? more" to match Go version
            ],
        }
    }

    /// Returns all key bindings organized into logical groups.
    ///
    /// Provides comprehensive help information with bindings grouped by
    /// functionality. The grouping helps users understand related actions
    /// and discover advanced features.
    ///
    /// # Binding Groups
    ///
    /// 1. **Cursor movement**: Up/down navigation
    /// 2. **Page navigation**: Page up/down, home/end
    /// 3. **Filtering**: Start filter, clear filter, accept
    /// 4. **Help and quit**: Show help, quit application
    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![
            // Column 1: Primary Navigation
            vec![
                &self.keymap.cursor_up,
                &self.keymap.cursor_down,
                &self.keymap.next_page,
                &self.keymap.prev_page,
                &self.keymap.go_to_start,
                &self.keymap.go_to_end,
            ],
            // Column 2: Filtering Actions
            vec![
                &self.keymap.filter,
                &self.keymap.clear_filter,
                &self.keymap.accept_filter,
                &self.keymap.cancel_filter,
            ],
            // Column 3: Help and Quit
            vec![
                &self.keymap.show_full_help,
                &self.keymap.close_full_help,
                &self.keymap.quit,
            ],
        ]
    }
}

// BubbleTeaModel implementation - integrates with bubbletea-rs runtime
impl<I: Item + Send + Sync + 'static> BubbleTeaModel for Model<I> {
    /// Initializes a new empty list model with default settings.
    ///
    /// This creates a new list with no items, using the default delegate
    /// and standard dimensions. This method is called by the BubbleTea
    /// runtime when the model is first created.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The initialized list model with default settings
    /// - `None` (no initial command to execute)
    ///
    /// # Default Configuration
    ///
    /// - Empty items list
    /// - `DefaultDelegate` for rendering
    /// - 80 columns × 24 rows dimensions
    /// - Default styling and key bindings
    fn init() -> (Self, Option<Cmd>) {
        let model = Self::new(vec![], defaultitem::DefaultDelegate::new(), 80, 24);
        (model, None)
    }

    /// Handles keyboard input and state updates.
    ///
    /// This method processes all user input and updates the list state accordingly.
    /// It implements different input handling modes based on the current filtering state:
    ///
    /// ## While Filtering (`FilterState::Filtering`)
    ///
    /// - **Escape**: Cancel filtering, return to previous state
    /// - **Enter**: Accept current filter, change to `FilterApplied` state
    /// - **Characters**: Add to filter text, update results in real-time
    /// - **Backspace**: Remove characters from filter
    /// - **Arrow keys**: Navigate filter input cursor position
    ///
    /// ## Normal Navigation (other states)
    ///
    /// - **Up/Down**: Move cursor through items with smooth viewport scrolling
    /// - **Page Up/Page Down**: Move cursor by one page (viewport height)
    /// - **Home/End**: Jump to first/last item
    /// - **/** : Start filtering mode
    /// - **Ctrl+C**: Clear any active filter
    ///
    /// # Viewport and Paginator Management
    ///
    /// The update method automatically:
    /// - Manages viewport scrolling to ensure the cursor remains visible
    /// - Synchronizes the paginator component to reflect the current page
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
                        self.filter_state = FilterState::FilterApplied;
                        self.filter_input.blur();
                        return None;
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        // Forward character input to the textinput component for proper handling.
                        // Creating a new KeyMsg preserves the original event context while ensuring
                        // the textinput receives all necessary information for features like cursor
                        // positioning, selection, and character encoding.
                        let textinput_msg = Box::new(KeyMsg {
                            key: KeyCode::Char(c),
                            modifiers: key_msg.modifiers,
                        }) as Msg;
                        self.filter_input.update(textinput_msg);
                        self.apply_filter();
                    }
                    crossterm::event::KeyCode::Backspace => {
                        // Forward backspace events to textinput for complete input handling.
                        // The textinput component manages cursor positioning, selection deletion,
                        // and other editing features that require coordinated state management.
                        let textinput_msg = Box::new(KeyMsg {
                            key: KeyCode::Backspace,
                            modifiers: key_msg.modifiers,
                        }) as Msg;
                        self.filter_input.update(textinput_msg);
                        self.apply_filter();
                    }
                    // Handle cursor movement within filter input
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
                    if self.is_cursor_at_viewport_top() {
                        // Page-turning behavior: move to last item of previous page
                        let items_per_view = self.calculate_items_per_view();
                        self.cursor -= 1;
                        self.viewport_start = self.cursor.saturating_sub(items_per_view - 1);
                    } else {
                        // Normal single-item navigation
                        self.cursor -= 1;
                        self.sync_viewport_with_cursor();
                    }
                }
            } else if self.keymap.cursor_down.matches(key_msg) {
                if self.cursor < self.len().saturating_sub(1) {
                    if self.is_cursor_at_viewport_bottom() {
                        // Page-turning behavior: move to first item of next page
                        self.cursor += 1;
                        self.viewport_start = self.cursor;
                    } else {
                        // Normal single-item navigation
                        self.cursor += 1;
                        self.sync_viewport_with_cursor();
                    }
                }
            } else if self.keymap.go_to_start.matches(key_msg) {
                self.cursor = 0;
                // Adjust viewport to show the beginning of the list when jumping to start.
                self.sync_viewport_with_cursor();
            } else if self.keymap.go_to_end.matches(key_msg) {
                self.cursor = self.len().saturating_sub(1);
                // Adjust viewport to show the end of the list when jumping to last item.
                self.sync_viewport_with_cursor();
            } else if self.keymap.next_page.matches(key_msg) {
                // Page Down: Move cursor forward by one page (viewport height).
                // This provides quick navigation through long lists.
                let items_len = self.len();
                if items_len > 0 {
                    self.cursor = (self.cursor + self.per_page).min(items_len - 1);
                    self.sync_viewport_with_cursor();
                }
            } else if self.keymap.prev_page.matches(key_msg) {
                // Page Up: Move cursor backward by one page (viewport height).
                // Saturating subtraction ensures we don't underflow.
                self.cursor = self.cursor.saturating_sub(self.per_page);
                self.sync_viewport_with_cursor();
            } else if self.keymap.filter.matches(key_msg) {
                self.filter_state = FilterState::Filtering;
                // Return focus command to enable cursor blinking in filter input
                return Some(self.filter_input.focus());
            } else if self.keymap.clear_filter.matches(key_msg) {
                self.filter_input.set_value("");
                self.filter_state = FilterState::Unfiltered;
                self.filtered_items.clear();
                self.cursor = 0;
                self.update_pagination();
            } else if self.keymap.show_full_help.matches(key_msg)
                || self.keymap.close_full_help.matches(key_msg)
            {
                self.help.show_all = !self.help.show_all;
                self.update_pagination(); // Recalculate layout since help height changes
            } else if self.keymap.quit.matches(key_msg) {
                return Some(bubbletea_rs::quit());
            } else if key_msg.key == crossterm::event::KeyCode::Enter {
                // Handle item selection
                if let Some(selected_item) = self.selected_item() {
                    // Get the original index for the delegate callback
                    let original_index = if self.filter_state == FilterState::Unfiltered {
                        self.cursor
                    } else if let Some(filtered_item) = self.filtered_items.get(self.cursor) {
                        filtered_item.index
                    } else {
                        return None;
                    };

                    // Call the delegate's on_select callback
                    if let Some(cmd) = self.delegate.on_select(original_index, selected_item) {
                        return Some(cmd);
                    }
                }
            }

            // Synchronize the paginator component with the current cursor position.
            // This calculation determines which "page" the cursor is on based on
            // items per page, ensuring the pagination indicator (dots) accurately
            // reflects the user's position in the list.
            if self.per_page > 0 {
                self.paginator.page = self.cursor / self.per_page;
            }
        }
        None
    }

    /// Renders the complete list view as a formatted string.
    ///
    /// This method combines all visual components of the list into a single
    /// string suitable for terminal display. The layout adapts based on the
    /// current filtering state and available content.
    ///
    /// # Returns
    ///
    /// A formatted string containing the complete list UI with ANSI styling codes.
    ///
    /// # Layout Structure
    ///
    /// The view consists of three vertically stacked sections:
    ///
    /// 1. **Header**: Title or filter input (depending on state)
    ///    - Normal: "List Title" or "List Title (filtered: N)"
    ///    - Filtering: "Filter: > user_input"
    ///
    /// 2. **Items**: The main content area showing visible items
    ///    - Styled according to the current delegate
    ///    - Shows "No items" message when empty
    ///    - Viewport-based rendering for large lists
    ///
    /// 3. **Footer**: Status and help information
    ///    - Status: "X/Y items" format
    ///    - Help: Context-sensitive key bindings
    ///
    /// # Performance
    ///
    /// The view method only renders items currently visible in the viewport,
    /// ensuring consistent performance regardless of total item count.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bubbletea_widgets::list::{Model, DefaultDelegate, DefaultItem};
    /// # use bubbletea_rs::Model as BubbleTeaModel;
    /// let list = Model::new(
    ///     vec![DefaultItem::new("Item 1", "Description")],
    ///     DefaultDelegate::new(),
    ///     80, 24
    /// );
    ///
    /// let output = list.view();
    /// // Contains formatted list with title, items, and status bar
    /// ```
    fn view(&self) -> String {
        let mut sections = Vec::new();

        // Header: Title or filter input
        let header = self.view_header();
        if !header.is_empty() {
            sections.push(header);
        }

        // Items: Main content area
        let items = self.view_items();
        if !items.is_empty() {
            sections.push(items);
        }

        // Spinner: Loading indicator
        if self.show_spinner {
            let spinner_view = self.spinner.view();
            if !spinner_view.is_empty() {
                sections.push(spinner_view);
            }
        }

        // Pagination: Page indicators
        if self.show_pagination && !self.is_empty() && self.paginator.total_pages > 1 {
            let pagination_view = self.paginator.view();
            if !pagination_view.is_empty() {
                let styled_pagination = self
                    .styles
                    .pagination_style
                    .clone()
                    .render(&pagination_view);
                sections.push(styled_pagination);
            }
        }

        // Footer: Status and help
        let footer = self.view_footer();
        if !footer.is_empty() {
            sections.push(footer);
        }

        sections.join("\n")
    }
}
