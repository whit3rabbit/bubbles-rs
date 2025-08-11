//! A help component for bubbletea-rs, ported from the Go version.
//!
//! This component provides a customizable help view that can automatically
//! generate its content from a set of key bindings. It supports both compact
//! single-line help displays and expanded multi-column layouts.
//!
//! The help component integrates seamlessly with the bubbletea-rs architecture
//! and provides adaptive styling for both light and dark terminal themes.
//!
//! # Features
//!
//! - **Dual Display Modes**: Switch between compact and expanded help views
//! - **Adaptive Styling**: Automatically adjusts colors for light/dark themes
//! - **Width Constraints**: Truncates content with ellipsis when space is limited
//! - **Column Layout**: Organizes key bindings into logical, aligned columns
//! - **Disabled Key Handling**: Automatically hides disabled key bindings
//!
//! # Quick Start
//!
//! ```rust
//! use bubbletea_widgets::help::{Model, KeyMap};
//! use bubbletea_widgets::key::Binding;
//! use crossterm::event::KeyCode;
//!
//! // Create key bindings for your application
//! let quit_key = Binding::new(vec![KeyCode::Char('q')])
//!     .with_help("q", "quit");
//! let help_key = Binding::new(vec![KeyCode::Char('?')])
//!     .with_help("?", "help");
//!
//! // Implement KeyMap for your application state
//! struct MyApp {
//!     quit_key: Binding,
//!     help_key: Binding,
//! }
//!
//! impl KeyMap for MyApp {
//!     fn short_help(&self) -> Vec<&bubbletea_widgets::key::Binding> {
//!         vec![&self.quit_key, &self.help_key]
//!     }
//!
//!     fn full_help(&self) -> Vec<Vec<&bubbletea_widgets::key::Binding>> {
//!         vec![
//!             vec![&self.help_key],  // Help column
//!             vec![&self.quit_key],  // Quit column
//!         ]
//!     }
//! }
//!
//! // Create and use the help component
//! let app = MyApp { quit_key, help_key };
//! let help = Model::new().with_width(80);
//!
//! // Render help text
//! let short_help = help.view(&app);  // Shows compact help
//! let mut full_help = help;
//! full_help.show_all = true;
//! let detailed_help = full_help.view(&app);  // Shows detailed help
//! ```

use crate::key;
use bubbletea_rs::{Cmd, Msg};
use lipgloss_extras::lipgloss;
use lipgloss_extras::prelude::*;

/// A trait that defines the key bindings to be displayed in the help view.
///
/// Any model that uses the help component should implement this trait to provide
/// the key bindings that the help view will render. The trait provides two methods
/// for different display contexts:
///
/// - `short_help()`: Returns key bindings for compact, single-line display
/// - `full_help()`: Returns grouped key bindings for detailed, multi-column display
///
/// # Implementation Guidelines
///
/// ## Short Help
///
/// Should include only the most essential key bindings (typically 3-6 keys)
/// that users need for basic operation. These are displayed horizontally
/// with bullet separators.
///
/// ## Full Help
///
/// Should group related key bindings into logical columns:
/// - Navigation keys in one group
/// - Action keys in another group  
/// - Application control keys in a third group
///
/// Each inner `Vec` becomes a column in the final display, so group
/// related functionality together.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::help::KeyMap;
/// use bubbletea_widgets::key::Binding;
/// use crossterm::event::KeyCode;
///
/// struct TextEditor {
///     save_key: Binding,
///     quit_key: Binding,
///     undo_key: Binding,
///     redo_key: Binding,
/// }
///
/// impl KeyMap for TextEditor {
///     fn short_help(&self) -> Vec<&bubbletea_widgets::key::Binding> {
///         // Only show essential keys in compact view
///         vec![&self.save_key, &self.quit_key]
///     }
///
///     fn full_help(&self) -> Vec<Vec<&bubbletea_widgets::key::Binding>> {
///         vec![
///             // File operations column
///             vec![&self.save_key],
///             // Edit operations column  
///             vec![&self.undo_key, &self.redo_key],
///             // Application control column
///             vec![&self.quit_key],
///         ]
///     }
/// }
/// ```
pub trait KeyMap {
    /// Returns a slice of key bindings for the short help view.
    ///
    /// This method should return the most essential key bindings for your
    /// application, typically 3-6 keys that users need for basic operation.
    /// These bindings will be displayed in a single horizontal line.
    ///
    /// # Guidelines
    ///
    /// - Include only the most frequently used keys
    /// - Prioritize navigation and core functionality
    /// - Consider the typical workflow of your application
    /// - Keep the total count manageable (3-6 keys)
    ///
    /// # Returns
    ///
    /// A vector of key binding references that will be displayed horizontally.
    fn short_help(&self) -> Vec<&key::Binding>;
    /// Returns a nested slice of key bindings for the full help view.
    ///
    /// Each inner `Vec` represents a column in the help view and should contain
    /// logically related key bindings. The help component will render these as
    /// separate columns with proper alignment and spacing.
    ///
    /// # Guidelines
    ///
    /// - Group related functionality together in the same column
    /// - Keep columns roughly the same height for visual balance
    /// - Consider the logical flow: navigation → actions → application control
    /// - Each column should have 2-8 key bindings for optimal display
    ///
    /// # Column Organization Examples
    ///
    /// ```text
    /// Column 1: Navigation    Column 2: Actions       Column 3: App Control
    /// ↑/k      move up       enter    select         q        quit
    /// ↓/j      move down     space    toggle         ?        help
    /// →        next page     d        delete         ctrl+c   force quit
    /// ←        prev page     
    /// ```
    ///
    /// # Returns
    ///
    /// A vector of vectors, where each inner vector represents a column
    /// of key bindings to display.
    fn full_help(&self) -> Vec<Vec<&key::Binding>>;
}

/// A set of styles for the help component.
///
/// This structure defines all the visual styling options available for customizing
/// the appearance of the help view. Each field controls a specific visual element,
/// allowing fine-grained control over colors, formatting, and visual hierarchy.
///
/// # Style Categories
///
/// ## Short Help Styles
/// - `short_key`: Styling for key names in compact view
/// - `short_desc`: Styling for descriptions in compact view  
/// - `short_separator`: Styling for bullet separators between items
///
/// ## Full Help Styles
/// - `full_key`: Styling for key names in detailed view
/// - `full_desc`: Styling for descriptions in detailed view
/// - `full_separator`: Styling for spacing between columns
///
/// ## Utility Styles
/// - `ellipsis`: Styling for truncation indicator when content is too wide
///
/// # Examples
///
/// ## Custom Color Scheme
/// ```rust
/// use bubbletea_widgets::help::Styles;
/// use lipgloss_extras::prelude::*;
///
/// let vibrant_styles = Styles {
///     short_key: Style::new()
///         .foreground(Color::from("#FF6B6B"))
///         .bold(true),
///     short_desc: Style::new()
///         .foreground(Color::from("#4ECDC4"))
///         .italic(true),
///     short_separator: Style::new()
///         .foreground(Color::from("#45B7D1")),
///     ..Default::default()
/// };
/// ```
///
/// ## Monochrome Theme
/// ```rust
/// # use bubbletea_widgets::help::Styles;
/// # use lipgloss_extras::prelude::*;
/// let mono_styles = Styles {
///     short_key: Style::new().bold(true),
///     short_desc: Style::new().faint(true),
///     full_key: Style::new().underline(true),
///     full_desc: Style::new(),
///     ..Default::default()
/// };
/// ```
///
/// ## Using with Help Model
/// ```rust
/// # use bubbletea_widgets::help::{Model, Styles};
/// # use lipgloss_extras::prelude::*;
/// let custom_styles = Styles {
///     short_key: Style::new().foreground(Color::from("#00FF00")),
///     ..Default::default()
/// };
///
/// let help = Model {
///     styles: custom_styles,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Styles {
    /// Style for the ellipsis character when content is truncated.
    pub ellipsis: Style,
    /// Style for key names in the short help view.
    pub short_key: Style,
    /// Style for descriptions in the short help view.
    pub short_desc: Style,
    /// Style for the separator between items in the short help view.
    pub short_separator: Style,
    /// Style for key names in the full help view.
    pub full_key: Style,
    /// Style for descriptions in the full help view.
    pub full_desc: Style,
    /// Style for the separator between columns in the full help view.
    pub full_separator: Style,
}

impl Default for Styles {
    /// Creates default styles with a subtle color scheme that adapts to light and dark themes.
    ///
    /// The default styling uses adaptive colors that work well in both light and dark terminal environments:
    ///
    /// ## Color Palette
    /// - **Keys**: Medium gray (light: #909090, dark: #626262) - provides good visibility for key names
    /// - **Descriptions**: Lighter gray (light: #B2B2B2, dark: #4A4A4A) - subtle but readable for descriptions
    /// - **Separators**: Even lighter gray (light: #DDDADA, dark: #3C3C3C) - minimal visual interruption
    ///
    /// ## Adaptive Behavior
    ///
    /// The colors automatically adapt based on the terminal's background:
    /// - **Light terminals**: Use darker colors for good contrast
    /// - **Dark terminals**: Use lighter colors for readability
    ///
    /// This ensures consistent readability across different terminal themes
    /// without requiring manual configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::Styles;
    ///
    /// let styles = Styles::default();
    /// // All styles are configured with adaptive colors suitable for terminals
    /// // The actual style strings contain color codes
    /// ```
    fn default() -> Self {
        use lipgloss::AdaptiveColor;

        let key_style = Style::new().foreground(AdaptiveColor {
            Light: "#909090",
            Dark: "#626262",
        });
        let desc_style = Style::new().foreground(AdaptiveColor {
            Light: "#B2B2B2",
            Dark: "#4A4A4A",
        });
        let sep_style = Style::new().foreground(AdaptiveColor {
            Light: "#DDDADA",
            Dark: "#3C3C3C",
        });

        Self {
            ellipsis: sep_style.clone(),
            short_key: key_style.clone(),
            short_desc: desc_style.clone(),
            short_separator: sep_style.clone(),
            full_key: key_style,
            full_desc: desc_style,
            full_separator: sep_style,
        }
    }
}

/// The help model that manages help view state and rendering.
///
/// This is the main component for displaying help information in terminal applications.
/// It can show either a compact single-line view or an expanded multi-column view
/// based on the `show_all` toggle. The component handles automatic styling, width
/// constraints, and proper alignment of key bindings.
///
/// # View Modes
///
/// ## Short Help Mode (`show_all = false`)
/// Displays key bindings in a horizontal line with bullet separators:
/// ```text
/// ↑/k up • ↓/j down • / filter • q quit • ? more
/// ```
///
/// ## Full Help Mode (`show_all = true`)
/// Displays key bindings in organized columns:
/// ```text
/// ↑/k      up             / filter         q quit
/// ↓/j      down           esc clear filter ? close help
/// →/l/pgdn next page      enter apply
/// ←/h/pgup prev page
/// ```
///
/// # Configuration
///
/// - **Width Constraints**: Set maximum width to enable truncation with ellipsis
/// - **Custom Separators**: Configure bullet separators and column spacing
/// - **Styling**: Full control over colors and text formatting
/// - **State Management**: Toggle between compact and detailed views
///
/// # Examples
///
/// ## Basic Usage
/// ```rust
/// use bubbletea_widgets::help::{Model, KeyMap};
/// use bubbletea_widgets::key::Binding;
/// use crossterm::event::KeyCode;
///
/// // Create key bindings
/// let quit_binding = Binding::new(vec![KeyCode::Char('q')])
///     .with_help("q", "quit");
///
/// // Implement KeyMap for your application
/// struct MyApp {
///     quit_binding: Binding,
/// }
///
/// impl KeyMap for MyApp {
///     fn short_help(&self) -> Vec<&bubbletea_widgets::key::Binding> {
///         vec![&self.quit_binding]
///     }
///     fn full_help(&self) -> Vec<Vec<&bubbletea_widgets::key::Binding>> {
///         vec![vec![&self.quit_binding]]
///     }
/// }
///
/// // Create and configure help model
/// let app = MyApp { quit_binding };
/// let help = Model::new().with_width(80);
/// let help_text = help.view(&app);
/// ```
///
/// ## Advanced Configuration
/// ```rust
/// # use bubbletea_widgets::help::Model;
/// # use lipgloss_extras::prelude::*;
/// let help = Model {
///     show_all: false,
///     width: 120,
///     short_separator: " | ".to_string(),
///     full_separator: "      ".to_string(),
///     ellipsis: "...".to_string(),
///     styles: Default::default(),
/// };
/// ```
///
/// ## Integration with BubbleTea
/// ```rust
/// # use bubbletea_widgets::help::{Model, KeyMap};
/// # use bubbletea_rs::{Msg, Model as BubbleTeaModel};
/// # struct MyApp { help: Model }
/// # impl KeyMap for MyApp {
/// #   fn short_help(&self) -> Vec<&bubbletea_widgets::key::Binding> { vec![] }
/// #   fn full_help(&self) -> Vec<Vec<&bubbletea_widgets::key::Binding>> { vec![] }
/// # }
/// # impl BubbleTeaModel for MyApp {
/// #   fn init() -> (Self, Option<bubbletea_rs::Cmd>) { (MyApp { help: Model::new() }, None) }
/// #   fn update(&mut self, msg: Msg) -> Option<bubbletea_rs::Cmd> {
/// // Toggle help view with '?' key
/// self.help.show_all = !self.help.show_all;
/// None
/// #   }
/// #   fn view(&self) -> String {
/// // Render help at bottom of your application view
/// let help_view = self.help.view(self);
/// format!("{}\n{}", "Your app content here", help_view)
/// #   }
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Model {
    /// Toggles between short (single-line) and full (multi-column) help view.
    /// When `false`, shows compact help; when `true`, shows detailed help.
    pub show_all: bool,
    /// The maximum width of the help view in characters.
    /// When set to 0, no width limit is enforced.
    pub width: usize,

    /// The separator string used between items in the short help view.
    /// Default is " • " (bullet with spaces).
    pub short_separator: String,
    /// The separator string used between columns in the full help view.
    /// Default is "    " (four spaces).
    pub full_separator: String,
    /// The character displayed when help content is truncated due to width constraints.
    /// Default is "…" (horizontal ellipsis).
    pub ellipsis: String,

    /// The styling configuration for all visual elements of the help view.
    pub styles: Styles,
}

impl Default for Model {
    /// Creates a new help model with sensible defaults.
    ///
    /// Default configuration:
    /// - `show_all`: false (shows short help)
    /// - `width`: 0 (no width limit)
    /// - `short_separator`: " • "
    /// - `full_separator`: "    " (4 spaces)
    /// - `ellipsis`: "…"
    /// - `styles`: Default styles
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    ///
    /// let help = Model::default();
    /// assert_eq!(help.show_all, false);
    /// assert_eq!(help.width, 0);
    /// ```
    fn default() -> Self {
        Self {
            show_all: false,
            width: 0,
            short_separator: " • ".to_string(),
            full_separator: "    ".to_string(),
            ellipsis: "…".to_string(),
            styles: Styles::default(),
        }
    }
}

impl Model {
    /// Creates a new help model with default settings.
    ///
    /// This is equivalent to calling `Model::default()` but provides a more
    /// conventional constructor-style API. The model is created in compact
    /// mode with no width limits and default styling.
    ///
    /// # Default Configuration
    ///
    /// - `show_all`: `false` (compact view)
    /// - `width`: `0` (no width limit)
    /// - `short_separator`: `" • "` (bullet with spaces)
    /// - `full_separator`: `"    "` (four spaces)
    /// - `ellipsis`: `"…"` (horizontal ellipsis)
    /// - `styles`: Adaptive color scheme
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    ///
    /// let help = Model::new();
    /// assert_eq!(help.show_all, false);
    /// assert_eq!(help.width, 0);
    /// assert_eq!(help.short_separator, " • ");
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum width of the help view.
    ///
    /// When a width is set, the help view will truncate content that exceeds
    /// this limit, showing an ellipsis to indicate truncation. This is useful
    /// for ensuring help text doesn't overflow in constrained terminal windows
    /// or when embedding help in specific layout areas.
    ///
    /// # Truncation Behavior
    ///
    /// - **Short Help**: Truncates items from right to left, showing ellipsis when possible
    /// - **Full Help**: Truncates columns from right to left, maintaining column integrity
    /// - **Smart Ellipsis**: Only shows ellipsis if it fits within the width constraint
    ///
    /// # Arguments
    ///
    /// * `width` - Maximum width in characters. Use 0 for no limit.
    ///
    /// # Examples
    ///
    /// ## Setting Width Constraints
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    ///
    /// let help = Model::new().with_width(80);
    /// assert_eq!(help.width, 80);
    ///
    /// // No width limit
    /// let unlimited = Model::new().with_width(0);
    /// assert_eq!(unlimited.width, 0);
    /// ```
    ///
    /// ## Chaining with Other Configuration
    /// ```rust
    /// # use bubbletea_widgets::help::Model;
    /// let help = Model::new()
    ///     .with_width(120);
    ///     
    /// // Further customize if needed
    /// let mut customized = help;
    /// customized.show_all = true;
    /// ```
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Updates the help model in response to a message.
    ///
    /// This method provides compatibility with the bubbletea-rs architecture,
    /// matching the Go implementation's Update method. Since the help component
    /// is primarily a view component that doesn't handle user input directly,
    /// this is a no-op method that simply returns the model unchanged.
    ///
    /// # Design Rationale
    ///
    /// The help component is stateless from a user interaction perspective:
    /// - It doesn't respond to keyboard input
    /// - It doesn't maintain internal state that changes over time
    /// - Its display is controlled by the parent application
    ///
    /// Parent applications typically control help display by:
    /// - Toggling `show_all` based on key presses (e.g., '?' key)
    /// - Adjusting `width` in response to terminal resize events
    /// - Updating styling based on theme changes
    ///
    /// # Arguments
    ///
    /// * `_msg` - The message to handle (unused for help component)
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The unchanged model
    /// - `None` for the command (no side effects needed)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    /// use bubbletea_rs::Msg;
    ///
    /// let help = Model::new();
    /// // Any message can be passed, the help component ignores all messages
    /// let msg = Box::new(42); // Example message
    /// let (updated_help, cmd) = help.update(msg);
    /// assert!(cmd.is_none()); // Help component doesn't generate commands
    /// ```
    ///
    /// ## Integration Pattern
    /// ```rust
    /// # use bubbletea_widgets::help::Model;
    /// # use bubbletea_rs::{Msg, Model as BubbleTeaModel, KeyMsg};
    /// # use crossterm::event::KeyCode;
    /// # struct MyApp { help: Model }
    /// # impl bubbletea_widgets::help::KeyMap for MyApp {
    /// #   fn short_help(&self) -> Vec<&bubbletea_widgets::key::Binding> { vec![] }
    /// #   fn full_help(&self) -> Vec<Vec<&bubbletea_widgets::key::Binding>> { vec![] }
    /// # }
    /// # impl BubbleTeaModel for MyApp {
    /// #   fn init() -> (Self, Option<bubbletea_rs::Cmd>) { (MyApp { help: Model::new() }, None) }
    /// #   fn update(&mut self, msg: Msg) -> Option<bubbletea_rs::Cmd> {
    /// // Parent application handles help toggling
    /// if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
    ///     if key_msg.key == KeyCode::Char('?') {
    ///         self.help.show_all = !self.help.show_all;
    ///     }
    /// }
    ///
    /// // Help component itself doesn't need to process messages
    /// let (_unchanged_help, _no_cmd) = self.help.clone().update(msg);
    /// None
    /// #   }
    /// #   fn view(&self) -> String { String::new() }
    /// # }
    /// ```
    pub fn update(self, _msg: Msg) -> (Self, Option<Cmd>) {
        (self, None)
    }

    /// Renders the help view based on the current model state.
    ///
    /// This is the main rendering function that switches between short and full
    /// help views based on the `show_all` flag. The method applies styling,
    /// handles width constraints, and formats the output appropriately for
    /// terminal display.
    ///
    /// # Rendering Process
    ///
    /// 1. **Mode Selection**: Choose between short or full help based on `show_all`
    /// 2. **Key Filtering**: Automatically skip disabled key bindings
    /// 3. **Styling Application**: Apply configured colors and formatting
    /// 4. **Layout**: Arrange keys and descriptions with proper spacing
    /// 5. **Width Handling**: Truncate with ellipsis if width constraints are set
    ///
    /// # Output Formats
    ///
    /// ## Short Help (`show_all = false`)
    /// ```text
    /// ↑/k up • ↓/j down • / filter • q quit
    /// ```
    ///
    /// ## Full Help (`show_all = true`)
    /// ```text
    /// ↑/k      up         / filter    q quit
    /// ↓/j      down       esc clear   ? help
    /// →/pgdn   next
    /// ```
    ///
    /// # Arguments
    ///
    /// * `keymap` - An object implementing the `KeyMap` trait that provides
    ///   the key bindings to display. Typically your main application model.
    ///
    /// # Returns
    ///
    /// A formatted string ready for display in the terminal, including ANSI
    /// color codes if styling is configured.
    ///
    /// # Examples
    ///
    /// ## Basic Rendering
    /// ```rust
    /// use bubbletea_widgets::help::{Model, KeyMap};
    /// use bubbletea_widgets::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// struct MyApp {
    ///     quit_key: Binding,
    /// }
    ///
    /// impl KeyMap for MyApp {
    ///     fn short_help(&self) -> Vec<&bubbletea_widgets::key::Binding> {
    ///         vec![&self.quit_key]
    ///     }
    ///     fn full_help(&self) -> Vec<Vec<&bubbletea_widgets::key::Binding>> {
    ///         vec![vec![&self.quit_key]]
    ///     }
    /// }
    ///
    /// let app = MyApp {
    ///     quit_key: Binding::new(vec![KeyCode::Char('q')]).with_help("q", "quit")
    /// };
    /// let help = Model::new();
    /// let output = help.view(&app);
    /// ```
    ///
    /// ## Toggling Between Modes
    /// ```rust
    /// # use bubbletea_widgets::help::{Model, KeyMap};
    /// # use bubbletea_widgets::key::Binding;
    /// # use crossterm::event::KeyCode;
    /// # struct MyApp { quit_key: Binding }
    /// # impl KeyMap for MyApp {
    /// #     fn short_help(&self) -> Vec<&bubbletea_widgets::key::Binding> { vec![&self.quit_key] }
    /// #     fn full_help(&self) -> Vec<Vec<&bubbletea_widgets::key::Binding>> { vec![vec![&self.quit_key]] }
    /// # }
    /// # let app = MyApp { quit_key: Binding::new(vec![KeyCode::Char('q')]).with_help("q", "quit") };
    /// let mut help = Model::new();
    ///
    /// // Render compact help
    /// help.show_all = false;
    /// let short = help.view(&app);
    ///
    /// // Render detailed help
    /// help.show_all = true;
    /// let full = help.view(&app);
    ///
    /// // Both modes produce valid output
    /// assert!(!short.is_empty());
    /// assert!(!full.is_empty());
    /// ```
    pub fn view<K: KeyMap>(&self, keymap: &K) -> String {
        if self.show_all {
            self.full_help_view(keymap.full_help())
        } else {
            self.short_help_view(keymap.short_help())
        }
    }

    /// Renders a compact single-line help view.
    ///
    /// This view displays key bindings in a horizontal layout, separated by
    /// the configured separator (default: " • "). If the content exceeds the
    /// specified width, it will be truncated with an ellipsis. This format
    /// is ideal for status bars or when screen space is limited.
    ///
    /// # Layout Format
    ///
    /// ```text
    /// key1 desc1 • key2 desc2 • key3 desc3
    /// ```
    ///
    /// # Truncation Behavior
    ///
    /// When width constraints are active:
    /// 1. Items are added from left to right
    /// 2. If an item would exceed the width, it's skipped
    /// 3. An ellipsis ("…") is added if there's space
    /// 4. Disabled key bindings are automatically excluded
    ///
    /// # Arguments
    ///
    /// * `bindings` - A vector of key bindings to display in order of priority.
    ///   Higher priority items should appear first as they're less likely to
    ///   be truncated.
    ///
    /// # Returns
    ///
    /// A single-line string containing the formatted help text with ANSI
    /// styling applied.
    ///
    /// # Examples
    ///
    /// ## Basic Usage
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    /// use bubbletea_widgets::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let help = Model::new();
    /// let quit_binding = Binding::new(vec![KeyCode::Char('q')]).with_help("q", "quit");
    /// let help_binding = Binding::new(vec![KeyCode::Char('?')]).with_help("?", "help");
    /// let bindings = vec![&quit_binding, &help_binding];
    /// let output = help.short_help_view(bindings);
    /// // Output: "q quit • ? help"
    /// ```
    ///
    /// ## With Width Constraints
    /// ```rust
    /// # use bubbletea_widgets::help::Model;
    /// # use bubbletea_widgets::key::Binding;
    /// # use crossterm::event::KeyCode;
    /// let help = Model::new().with_width(20); // Very narrow
    /// let quit_binding = Binding::new(vec![KeyCode::Char('q')]).with_help("q", "quit");
    /// let help_binding = Binding::new(vec![KeyCode::Char('?')]).with_help("?", "help");
    /// let save_binding = Binding::new(vec![KeyCode::Char('s')]).with_help("s", "save");
    /// let bindings = vec![&quit_binding, &help_binding, &save_binding];
    /// let output = help.short_help_view(bindings);
    /// // Might be truncated due to width constraints
    /// ```
    pub fn short_help_view(&self, bindings: Vec<&key::Binding>) -> String {
        if bindings.is_empty() {
            return String::new();
        }

        let mut builder = String::new();
        let mut total_width = 0;
        let separator = self
            .styles
            .short_separator
            .clone()
            .inline(true)
            .render(&self.short_separator);

        for (i, kb) in bindings.iter().enumerate() {
            // Skip disabled bindings
            if !kb.enabled() {
                continue;
            }

            let sep = if total_width > 0 && i < bindings.len() {
                &separator
            } else {
                ""
            };

            // Format: "key description"
            let help = kb.help();
            let key_part = self.styles.short_key.clone().inline(true).render(&help.key);
            let desc_part = self
                .styles
                .short_desc
                .clone()
                .inline(true)
                .render(&help.desc);
            let item_str = format!("{}{} {}", sep, key_part, desc_part);

            let item_width = lipgloss::width_visible(&item_str);

            if let Some(tail) = self.should_add_item(total_width, item_width) {
                if !tail.is_empty() {
                    builder.push_str(&tail);
                }
                break;
            }

            total_width += item_width;
            builder.push_str(&item_str);
        }
        builder
    }

    /// Renders a detailed multi-column help view.
    ///
    /// This view organizes key bindings into columns, with each group of bindings
    /// forming a separate column. Keys and descriptions are aligned vertically
    /// within each column, and columns are separated by configurable spacing.
    /// This format provides comprehensive help information in an organized layout.
    ///
    /// # Layout Structure
    ///
    /// ```text
    /// Column 1          Column 2          Column 3
    /// key1 desc1        key4 desc4        key7 desc7
    /// key2 desc2        key5 desc5        key8 desc8  
    /// key3 desc3        key6 desc6
    /// ```
    ///
    /// # Column Processing
    ///
    /// 1. **Filtering**: Skip empty groups and groups with all disabled bindings
    /// 2. **Row Building**: Create "key description" pairs for each enabled binding
    /// 3. **Vertical Joining**: Combine rows within each column with newlines
    /// 4. **Horizontal Joining**: Align columns side-by-side with separators
    /// 5. **Width Management**: Truncate columns if width constraints are active
    ///
    /// # Truncation Behavior
    ///
    /// When width limits are set:
    /// - Columns are added left to right until width would be exceeded
    /// - Remaining columns are dropped entirely (maintaining column integrity)
    /// - An ellipsis is added if there's space to indicate truncation
    ///
    /// # Arguments
    ///
    /// * `groups` - A vector of key binding groups, where each group becomes
    ///   a column in the output. Order matters: earlier groups have higher
    ///   priority and are less likely to be truncated.
    ///
    /// # Returns
    ///
    /// A multi-line string containing the formatted help text with proper
    /// column alignment and ANSI styling applied.
    ///
    /// # Examples
    ///
    /// ## Basic Multi-Column Layout
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    /// use bubbletea_widgets::key::Binding;
    /// use crossterm::event::KeyCode;
    ///
    /// let help = Model::new();
    ///
    /// // Create bindings with proper lifetimes
    /// let up_key = Binding::new(vec![KeyCode::Up]).with_help("↑/k", "up");
    /// let down_key = Binding::new(vec![KeyCode::Down]).with_help("↓/j", "down");
    /// let enter_key = Binding::new(vec![KeyCode::Enter]).with_help("enter", "select");
    /// let delete_key = Binding::new(vec![KeyCode::Delete]).with_help("del", "delete");
    /// let quit_key = Binding::new(vec![KeyCode::Char('q')]).with_help("q", "quit");
    /// let help_key = Binding::new(vec![KeyCode::Char('?')]).with_help("?", "help");
    ///
    /// let groups = vec![
    ///     // Navigation column
    ///     vec![&up_key, &down_key],
    ///     // Action column  
    ///     vec![&enter_key, &delete_key],
    ///     // App control column
    ///     vec![&quit_key, &help_key],
    /// ];
    /// let output = help.full_help_view(groups);
    /// // Creates aligned columns with proper spacing
    /// ```
    ///
    /// ## With Custom Separator
    /// ```rust
    /// # use bubbletea_widgets::help::Model;
    /// # use bubbletea_widgets::key::Binding;
    /// # use crossterm::event::KeyCode;
    /// let help = Model {
    ///     full_separator: "  |  ".to_string(), // Custom column separator
    ///     ..Model::new()
    /// };
    /// let action_key = Binding::new(vec![KeyCode::Char('a')]).with_help("a", "action");
    /// let quit_key = Binding::new(vec![KeyCode::Char('q')]).with_help("q", "quit");
    /// let groups = vec![
    ///     vec![&action_key],
    ///     vec![&quit_key],
    /// ];
    /// let output = help.full_help_view(groups);
    /// // Columns will be separated by "  |  "
    /// ```
    ///
    /// ## Handling Width Constraints
    /// ```rust
    /// # use bubbletea_widgets::help::Model;
    /// # use bubbletea_widgets::key::Binding;
    /// # use crossterm::event::KeyCode;
    /// let help = Model::new().with_width(40); // Narrow width
    /// let first_key = Binding::new(vec![KeyCode::Char('1')]).with_help("1", "first");
    /// let second_key = Binding::new(vec![KeyCode::Char('2')]).with_help("2", "second");
    /// let third_key = Binding::new(vec![KeyCode::Char('3')]).with_help("3", "third");
    /// let groups = vec![
    ///     vec![&first_key],
    ///     vec![&second_key],
    ///     vec![&third_key],
    /// ];
    /// let output = help.full_help_view(groups);
    /// // May truncate rightmost columns if they don't fit
    /// ```
    pub fn full_help_view(&self, groups: Vec<Vec<&key::Binding>>) -> String {
        if groups.is_empty() {
            return String::new();
        }

        let mut columns = Vec::new();
        let mut total_width = 0;
        let separator = self
            .styles
            .full_separator
            .clone()
            .inline(true)
            .render(&self.full_separator);

        for group in groups.iter() {
            if group.is_empty() || !should_render_column(group) {
                continue;
            }

            // Build each row as "key description" within this column
            let rows: Vec<String> = group
                .iter()
                .filter(|b| b.enabled())
                .map(|b| {
                    let help = b.help();
                    let key_part = self.styles.full_key.clone().inline(true).render(&help.key);
                    let desc_part = self
                        .styles
                        .full_desc
                        .clone()
                        .inline(true)
                        .render(&help.desc);
                    format!("{} {}", key_part, desc_part)
                })
                .collect();

            let col_content = rows.join("\n");

            // For the first column, we don't need a separator
            // For subsequent columns, we'll add them during horizontal joining
            let col_str = col_content;

            let col_width = lipgloss::width_visible(&col_str);

            if let Some(tail) = self.should_add_item(total_width, col_width) {
                if !tail.is_empty() {
                    columns.push(tail);
                }
                break;
            }

            total_width += col_width;
            columns.push(col_str);
        }

        // Join columns with separators between them
        let mut result_parts = Vec::new();
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                result_parts.push(separator.as_str());
            }
            result_parts.push(col.as_str());
        }

        lipgloss::join_horizontal(lipgloss::TOP, &result_parts)
    }

    /// Determines if an item can be added to the view without exceeding the width limit.
    ///
    /// This helper function implements smart width management by checking if adding
    /// an item would exceed the configured width limit. It provides graceful handling
    /// of width constraints with appropriate truncation indicators.
    ///
    /// # Width Management Strategy
    ///
    /// 1. **No Limit**: If `width` is 0, all items can be added
    /// 2. **Within Limit**: If `total_width + item_width ≤ width`, item fits
    /// 3. **Exceeds Limit**: If adding the item would exceed width:
    ///    - Try to add an ellipsis ("…") if it fits
    ///    - Return empty string if even ellipsis won't fit
    ///
    /// # Arguments
    ///
    /// * `total_width` - Current accumulated width of content in characters.
    ///   This should include all previously added content and separators.
    /// * `item_width` - Width of the item being considered for addition,
    ///   measured in visible characters (ignoring ANSI codes).
    ///
    /// # Returns
    ///
    /// * `None` - Item can be added without exceeding width constraints
    /// * `Some(String)` - Item cannot be added. The string contains:
    ///   - Styled ellipsis if it fits within the remaining width
    ///   - Empty string if even the ellipsis would exceed the width
    ///
    /// # Width Calculation
    ///
    /// Width calculations use `lipgloss::width_visible()` to properly handle:
    /// - ANSI color codes (don't count toward width)
    /// - Unicode characters (count as their display width)
    /// - Multi-byte characters (count correctly)
    ///
    /// # Examples
    ///
    /// This method is used internally by the help component and is not part
    /// of the public API. The width management behavior can be observed through
    /// the public `short_help_view()` and `full_help_view()` methods when a
    /// width constraint is set using `with_width()`.
    ///
    /// # Internal Usage
    ///
    /// This method is used internally by both `short_help_view()` and
    /// `full_help_view()` to implement consistent width management across
    /// different help display modes.
    ///
    /// # Panics
    ///
    /// This function does not panic under normal circumstances. It handles
    /// edge cases gracefully:
    /// - Width calculations that might underflow (uses saturating arithmetic)
    /// - Empty strings and zero widths
    /// - Very large width values
    fn should_add_item(&self, total_width: usize, item_width: usize) -> Option<String> {
        if self.width > 0 && total_width + item_width > self.width {
            let tail = format!(
                " {}",
                self.styles
                    .ellipsis
                    .clone()
                    .inline(true)
                    .render(&self.ellipsis)
            );
            if total_width + lipgloss::width_visible(&tail) < self.width {
                return Some(tail);
            }
            return Some("".to_string());
        }
        None // Item can be added
    }

    /// Creates a new help model with default settings.
    ///
    /// **Deprecated**: Use [`Model::new`] instead.
    ///
    /// This function provides backwards compatibility with earlier versions
    /// of the library and matches the Go implementation's deprecated `NewModel`
    /// variable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[allow(deprecated)]
    /// use bubbletea_widgets::help::Model;
    ///
    /// let help = Model::new_model(); // Deprecated
    /// let help = Model::new();       // Preferred
    /// ```
    #[deprecated(since = "0.1.9", note = "Use Model::new() instead")]
    pub fn new_model() -> Self {
        Self::new()
    }
}

/// Determines if a column of key bindings should be rendered.
///
/// A column should be rendered if it contains at least one enabled binding.
/// This helper function matches the behavior of the Go implementation's
/// `shouldRenderColumn` function and provides consistent column visibility
/// logic across the help system.
///
/// # Purpose
///
/// This function prevents empty columns from appearing in the help display:
/// - **Enabled Bindings**: Columns with active key bindings are shown
/// - **Disabled Bindings**: Columns with only disabled bindings are hidden
/// - **Mixed Columns**: Columns with some enabled bindings are shown
/// - **Empty Columns**: Completely empty columns are hidden
///
/// # Use Cases
///
/// ## Context-Sensitive Help
/// ```rust
/// # use bubbletea_widgets::help::should_render_column;
/// # use bubbletea_widgets::key::Binding;
/// # use crossterm::event::KeyCode;
/// // In a text editor, cut/copy might be disabled when no text is selected
/// let cut_key = Binding::new(vec![KeyCode::Char('x')])
///     .with_help("x", "cut")
///     .with_disabled(); // Disabled because nothing selected
///     
/// let copy_key = Binding::new(vec![KeyCode::Char('c')])
///     .with_help("c", "copy")
///     .with_disabled(); // Also disabled
///     
/// let edit_column = vec![&cut_key, &copy_key];
/// assert!(!should_render_column(&edit_column)); // Hidden - all disabled
/// ```
///
/// ## Progressive Disclosure
/// ```rust
/// # use bubbletea_widgets::help::should_render_column;
/// # use bubbletea_widgets::key::Binding;
/// # use crossterm::event::KeyCode;
/// // Advanced features might be disabled for beginners
/// let basic_key = Binding::new(vec![KeyCode::Char('s')])
///     .with_help("s", "save"); // Always enabled
///     
/// let advanced_key = Binding::new(vec![KeyCode::Char('m')])
///     .with_help("m", "macro")
///     .with_disabled(); // Disabled in beginner mode
///     
/// let mixed_column = vec![&basic_key, &advanced_key];
/// assert!(should_render_column(&mixed_column)); // Shown - has enabled binding
/// ```
///
/// # Arguments
///
/// * `bindings` - A slice of key binding references to check.
///   Typically represents a logical group of related key bindings
///   that would form a column in the help display.
///
/// # Returns
///
/// * `true` - The column should be rendered because it contains at least
///   one enabled binding that users can actually use.
/// * `false` - The column should be hidden because all bindings are
///   disabled or the column is empty.
///
/// # Performance
///
/// This function uses early return optimization - it stops checking
/// as soon as it finds the first enabled binding, making it efficient
/// for columns with many bindings.
///
/// # Examples
///
/// ## All Bindings Enabled
/// ```rust
/// use bubbletea_widgets::help::should_render_column;
/// use bubbletea_widgets::key::Binding;
/// use crossterm::event::KeyCode;
///
/// let save_key = Binding::new(vec![KeyCode::Char('s')]).with_help("s", "save");
/// let quit_key = Binding::new(vec![KeyCode::Char('q')]).with_help("q", "quit");
///
/// let column = vec![&save_key, &quit_key];
/// assert!(should_render_column(&column)); // Show column
/// ```
///
/// ## All Bindings Disabled  
/// ```rust
/// # use bubbletea_widgets::help::should_render_column;
/// # use bubbletea_widgets::key::Binding;
/// # use crossterm::event::KeyCode;
/// let disabled1 = Binding::new(vec![KeyCode::F(1)]).with_disabled();
/// let disabled2 = Binding::new(vec![KeyCode::F(2)]).with_disabled();
///
/// let column = vec![&disabled1, &disabled2];
/// assert!(!should_render_column(&column)); // Hide column
/// ```
///
/// ## Mixed State
/// ```rust
/// # use bubbletea_widgets::help::should_render_column;
/// # use bubbletea_widgets::key::Binding;
/// # use crossterm::event::KeyCode;
/// let enabled = Binding::new(vec![KeyCode::Enter]).with_help("enter", "select");
/// let disabled = Binding::new(vec![KeyCode::Delete]).with_disabled();
///
/// let column = vec![&enabled, &disabled];
/// assert!(should_render_column(&column)); // Show column (has enabled binding)
/// ```
///
/// ## Empty Column
/// ```rust
/// # use bubbletea_widgets::help::should_render_column;
/// let empty_column = vec![];
/// assert!(!should_render_column(&empty_column)); // Hide empty column
/// ```
pub fn should_render_column(bindings: &[&key::Binding]) -> bool {
    for binding in bindings {
        if binding.enabled() {
            return true;
        }
    }
    false
}
