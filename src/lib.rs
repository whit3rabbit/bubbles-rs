#![warn(missing_docs)]
#![doc(html_root_url = "https://docs.rs/bubbletea-widgets/")]

//! # bubbletea-widgets
//!
//! A Rust port of the Go library [charmbracelet/bubbles](https://github.com/charmbracelet/bubbles),
//! providing reusable TUI components for building terminal applications with [bubbletea-rs](https://github.com/joshka/bubbletea-rs).
//!
//! [![Crates.io](https://img.shields.io/crates/v/bubbletea-widgets.svg)](https://crates.io/crates/bubbletea-widgets)
//! [![Documentation](https://docs.rs/bubbletea-widgets/badge.svg)](https://docs.rs/bubbletea-widgets)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
//!
//! ## Overview
//!
//! bubbletea-widgets offers a collection of common terminal UI components that can be easily integrated
//! into bubbletea-rs applications. Each component follows the Elm Architecture pattern with
//! `init()`, `update()`, and `view()` methods, providing a consistent and predictable API
//! for building complex terminal user interfaces.
//!
//! ## Features
//!
//! - **Type-safe key bindings** with comprehensive key combination support
//! - **Focus management** system for keyboard navigation between components
//! - **Responsive design** with automatic width/height handling
//! - **Theming support** through customizable styles
//! - **Go compatibility** for easy migration from charmbracelet/bubbles
//! - **Performance optimized** with efficient rendering and state management
//!
//! ## Components
//!
//! - **Input Components**: `TextInput`, `TextArea`, `FilePicker`
//! - **Display Components**: `List`, `Table`, `Progress`, `Spinner`, `Help`
//! - **Utility Components**: `Cursor`, `Viewport`, `Paginator`, `Timer`, `Stopwatch`
//!
//! ## Focus Management
//!
//! All components implement the `Component` trait which provides standardized focus management:
//!
//! ```rust
//! use bubbletea_widgets::prelude::*;
//! use bubbletea_rs::Cmd;
//!
//! fn handle_focus<T: Component>(component: &mut T) {
//!     let _cmd: Option<Cmd> = component.focus();
//!     assert!(component.focused());
//!     component.blur();
//!     assert!(!component.focused());
//! }
//!
//! // Example with a text area (implements Component trait)
//! let mut textarea = textarea_new();
//! handle_focus(&mut textarea);
//! ```
//!
//! ## Key Bindings
//!
//! Components use the type-safe key binding system from the `key` module:
//!
//! ```rust
//! use bubbletea_widgets::key::{Binding, KeyMap};
//! use crossterm::event::{KeyCode, KeyModifiers};
//!
//! // Create key bindings
//! let confirm = Binding::new(vec![KeyCode::Enter])
//!     .with_help("enter", "Confirm selection");
//!
//! let save = Binding::new(vec![(KeyCode::Char('s'), KeyModifiers::CONTROL)])
//!     .with_help("ctrl+s", "Save file");
//!
//! // Implement KeyMap for your component
//! struct MyKeyMap {
//!     confirm: Binding,
//!     save: Binding,
//! }
//!
//! impl KeyMap for MyKeyMap {
//!     fn short_help(&self) -> Vec<&Binding> {
//!         vec![&self.confirm, &self.save]
//!     }
//!
//!     fn full_help(&self) -> Vec<Vec<&Binding>> {
//!         vec![
//!             vec![&self.confirm],
//!             vec![&self.save],
//!         ]
//!     }
//! }
//! ```
//!
//! ## Integration with bubbletea-rs
//!
//! Components are designed to work seamlessly with bubbletea-rs models:
//!
//! ```rust
//! use bubbletea_widgets::prelude::*;
//! use bubbletea_rs::{Model, Cmd, Msg};
//!
//! struct App {
//!     input: TextInput,
//! }
//!
//! impl Model for App {
//!     fn init() -> (Self, Option<Cmd>) {
//!         let mut input = textinput_new();
//!         let focus_cmd = input.focus();
//!         (Self { input }, Some(focus_cmd))
//!     }
//!
//!     fn update(&mut self, msg: Msg) -> Option<Cmd> {
//!         // Handle component updates
//!         if let Some(cmd) = self.input.update(msg) {
//!             return Some(cmd);
//!         }
//!
//!         // Handle other messages
//!         match msg {
//!             // Your app-specific message handling
//!             _ => None,
//!         }
//!     }
//!
//!     fn view(&self) -> String {
//!         format!("Enter text: {}\n{}", self.input.view(), "Press Ctrl+C to quit")
//!     }
//! }
//! ```
//!
//! ## Quick Start
//!
//! Add bubbletea-widgets to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! bubbletea-widgets = "0.1.0"
//! bubbletea-rs = "0.0.6"
//! crossterm = "0.27"
//! ```
//!
//! For convenience, you can import the prelude:
//!
//! ```rust
//! use bubbletea_widgets::prelude::*;
//! ```
//!
//! ## Component Overview
//!
//! | Component | Description | Use Case |
//! |-----------|-------------|----------|
//! | `TextInput` | Single-line text input | Forms, search boxes |
//! | `TextArea` | Multi-line text editor | Code editing, long text |
//! | `List` | Scrollable item list | Menus, file browsers |
//! | `Table` | Tabular data display | Data tables, spreadsheets |
//! | `Progress` | Progress bar with animation | Loading indicators |
//! | `Spinner` | Animated loading spinner | Background operations |
//! | `Help` | Key binding help display | User guidance |
//! | `FilePicker` | File system navigator | File selection |
//! | `Timer` | Countdown timer | Time-based operations |
//! | `Stopwatch` | Elapsed time tracker | Performance monitoring |

pub mod cursor;
pub mod filepicker;
pub mod help;
pub mod key;
pub mod list;
pub mod paginator;
pub mod progress;
pub mod spinner;
pub mod stopwatch;
pub mod table;
pub mod textarea;
pub mod textinput;
pub mod timer;
pub mod viewport;

use bubbletea_rs::Cmd;

/// Core trait for components that support focus management.
///
/// This trait provides a standardized interface for managing keyboard focus
/// across all bubbletea-widgets components. Components that implement this trait can
/// participate in focus management systems and provide consistent behavior
/// for keyboard navigation.
///
/// ## Focus States
///
/// - **Focused**: The component can receive keyboard input and should visually
///   indicate its active state
/// - **Blurred**: The component cannot receive keyboard input and should
///   display in an inactive state
///
/// ## Implementation Guidelines
///
/// When implementing this trait:
/// - `focus()` should set the component's focused state and may return a command
///   for initialization (e.g., starting a cursor blink timer)
/// - `blur()` should unset the focused state and clean up any focus-related state
/// - `focused()` should return the current focus state consistently
///
/// ## Examples
///
/// ### Basic Usage
///
/// ```rust
/// use bubbletea_widgets::prelude::*;
///
/// let mut input = textinput_new();
/// assert!(!input.focused());
///
/// input.focus();
/// assert!(input.focused());
///
/// input.blur();
/// assert!(!input.focused());
/// ```
///
/// ### Focus Management in Applications
///
/// ```rust
/// use bubbletea_widgets::prelude::*;
/// use bubbletea_rs::Cmd;
///
/// struct App {
///     input: TextInput,
///     textarea: TextArea,
///     focused_component: usize,
/// }
///
/// impl App {
///     fn focus_next(&mut self) -> Option<Cmd> {
///         // Blur current component
///         match self.focused_component {
///             0 => self.input.blur(),
///             1 => self.textarea.blur(),
///             _ => {}
///         }
///
///         // Focus next component
///         self.focused_component = (self.focused_component + 1) % 2;
///         match self.focused_component {
///             0 => Some(self.input.focus()),
///             1 => self.textarea.focus(),
///             _ => None,
///         }
///     }
/// }
/// ```
pub trait Component {
    /// Sets the component to focused state.
    ///
    /// This method should update the component's internal state to indicate
    /// that it can receive keyboard input. It may return a command for
    /// initialization tasks like starting timers or triggering redraws.
    ///
    /// # Returns
    ///
    /// An optional command to be executed by the bubbletea runtime. Common
    /// use cases include starting cursor blink timers or triggering immediate
    /// redraws to show the focus state change.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::prelude::*;
    ///
    /// let mut input = textinput_new();
    /// let cmd = input.focus();
    /// assert!(input.focused());
    /// // cmd may contain a cursor blink timer start command
    /// ```
    fn focus(&mut self) -> Option<Cmd>;

    /// Sets the component to blurred (unfocused) state.
    ///
    /// This method should update the component's internal state to indicate
    /// that it cannot receive keyboard input. It should clean up any
    /// focus-related resources like stopping timers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::prelude::*;
    ///
    /// let mut input = textinput_new();
    /// input.focus();
    /// assert!(input.focused());
    ///
    /// input.blur();
    /// assert!(!input.focused());
    /// ```
    fn blur(&mut self);

    /// Returns the current focus state of the component.
    ///
    /// # Returns
    ///
    /// `true` if the component is currently focused and can receive keyboard
    /// input, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::prelude::*;
    ///
    /// let mut input = textinput_new();
    /// assert!(!input.focused()); // Initially unfocused
    ///
    /// input.focus();
    /// assert!(input.focused()); // Now focused
    /// ```
    fn focused(&self) -> bool;
}

pub use cursor::Model as Cursor;
pub use filepicker::Model as FilePicker;
pub use help::Model as HelpModel;
pub use key::{
    matches, matches_binding, new_binding, with_disabled, with_help, with_keys, Binding,
    Help as KeyHelp, KeyMap, KeyPress,
};
pub use list::Model as List;
pub use list::{
    DefaultDelegate as ListDefaultDelegate, DefaultItem as ListDefaultItem,
    DefaultItemStyles as ListDefaultItemStyles, FilterState, FilterStateInfo, ListKeyMap,
    ListStyles,
};
pub use paginator::Model as Paginator;
pub use progress::Model as Progress;
pub use spinner::{
    new as spinner_new, with_spinner, with_style, Model as Spinner, SpinnerOption,
    TickMsg as SpinnerTickMsg, DOT, ELLIPSIS, GLOBE, HAMBURGER, JUMP, LINE, METER, MINI_DOT,
    MONKEY, MOON, POINTS, PULSE,
};
pub use stopwatch::Model as Stopwatch;
pub use table::Model as Table;
pub use textarea::{
    default_styles as textarea_default_styles, new as textarea_new, LineInfo, Model as TextArea,
    PasteErrMsg as TextAreaPasteErrMsg, PasteMsg as TextAreaPasteMsg,
};
pub use textinput::{
    blink, default_key_map as textinput_default_key_map, new as textinput_new, paste, EchoMode,
    KeyMap as TextInputKeyMap, Model as TextInput, PasteErrMsg, PasteMsg, ValidateFunc,
};
pub use timer::{
    new as timer_new, new_with_interval as timer_new_with_interval, Model as Timer,
    StartStopMsg as TimerStartStopMsg, TickMsg as TimerTickMsg, TimeoutMsg as TimerTimeoutMsg,
};
pub use viewport::Model as Viewport;

/// Prelude module for convenient imports.
///
/// This module re-exports the most commonly used types and functions from
/// bubbletea-widgets, allowing users to import everything they need with a single
/// `use` statement.
///
/// # Usage
///
/// Instead of importing each component individually:
///
/// ```rust
/// use bubbletea_widgets::{TextInput, List, Spinner, Component};
/// use bubbletea_widgets::key::{Binding, KeyMap};
/// ```
///
/// You can use the prelude:
///
/// ```rust
/// use bubbletea_widgets::prelude::*;
/// ```
///
/// # What's Included
///
/// The prelude includes:
/// - All component types (`TextInput`, `List`, `Table`, etc.)
/// - The `Component` trait for focus management
/// - Key binding types and functions (`Binding`, `KeyMap`, etc.)
/// - Utility types and commonly used enums
/// - Constructor functions for components
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::prelude::*;
/// use bubbletea_rs::{Model, Cmd, Msg};
///
/// struct App {
///     input: TextInput,
///     list: List<ListDefaultItem>,
/// }
///
/// impl Model for App {
///     fn init() -> (Self, Option<Cmd>) {
///         let mut input = textinput_new();
///         let focus_cmd = input.focus();
///         
///         let delegate = ListDefaultDelegate::new();
///         let items = vec![
///             ListDefaultItem::new("Item 1", "First item"),
///             ListDefaultItem::new("Item 2", "Second item"),
///         ];
///         let list = List::new(items, delegate, 80, 10);
///         
///         (Self { input, list }, Some(focus_cmd))
///     }
///
///     fn update(&mut self, msg: Msg) -> Option<Cmd> {
///         // Handle updates...
///         None
///     }
///
///     fn view(&self) -> String {
///         format!("{}\\n{}", self.input.view(), self.list.view())
///     }
/// }
/// ```
pub mod prelude {
    pub use crate::cursor::Model as Cursor;
    pub use crate::help::Model as HelpModel;
    pub use crate::key::{
        matches, matches_binding, new_binding, with_disabled, with_help, with_keys, Binding,
        Help as KeyHelp, KeyMap, KeyPress,
    };
    pub use crate::list::Model as List;
    pub use crate::list::{
        DefaultDelegate as ListDefaultDelegate, DefaultItem as ListDefaultItem,
        DefaultItemStyles as ListDefaultItemStyles, FilterState, FilterStateInfo, ListKeyMap,
        ListStyles,
    };
    pub use crate::paginator::Model as Paginator;
    pub use crate::progress::Model as Progress;
    pub use crate::spinner::{
        new as spinner_new, with_spinner, with_style, Model as Spinner, SpinnerOption,
        TickMsg as SpinnerTickMsg, DOT, ELLIPSIS, GLOBE, HAMBURGER, JUMP, LINE, METER, MINI_DOT,
        MONKEY, MOON, POINTS, PULSE,
    };
    pub use crate::table::Model as Table;
    pub use crate::textarea::{
        default_styles as textarea_default_styles, new as textarea_new, LineInfo,
        Model as TextArea, PasteErrMsg as TextAreaPasteErrMsg, PasteMsg as TextAreaPasteMsg,
    };
    pub use crate::textinput::{
        blink, default_key_map as textinput_default_key_map, new as textinput_new, paste, EchoMode,
        KeyMap as TextInputKeyMap, Model as TextInput, PasteErrMsg, PasteMsg, ValidateFunc,
    };
    pub use crate::timer::{
        new as timer_new, new_with_interval as timer_new_with_interval, Model as Timer,
        StartStopMsg as TimerStartStopMsg, TickMsg as TimerTickMsg, TimeoutMsg as TimerTimeoutMsg,
    };
    pub use crate::viewport::Model as Viewport;
    pub use crate::Component;
}
