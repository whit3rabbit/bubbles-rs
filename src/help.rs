//! A help component for bubbletea-rs, ported from the Go version.
//!
//! This component provides a customizable help view that can automatically
//! generate its content from a set of key bindings.

use crate::key;
use lipgloss_extras::prelude::*;
use lipgloss_extras::lipgloss;

/// A trait that defines the key bindings to be displayed in the help view.
///
/// Any model that uses the help component should implement this trait to provide
/// the key bindings that the help view will render.
pub trait KeyMap {
    /// Returns a slice of key bindings for the short help view.
    fn short_help(&self) -> Vec<&key::Binding>;
    /// Returns a nested slice of key bindings for the full help view.
    /// Each inner slice represents a column in the help view.
    fn full_help(&self) -> Vec<Vec<&key::Binding>>;
}

/// A set of styles for the help component.
///
/// This structure defines all the visual styling options available for customizing
/// the appearance of the help view. Each field controls a specific visual element.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::help::Styles;
/// use lipgloss_extras::prelude::*;
///
/// let custom_styles = Styles {
///     short_key: Style::new().foreground(Color::from("#FF6B6B")),
///     short_desc: Style::new().foreground(Color::from("#4ECDC4")),
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
    /// Creates default styles with a subtle color scheme.
    ///
    /// The default styling uses muted colors that work well in most terminal environments:
    /// - Keys are styled in a medium gray (#909090)
    /// - Descriptions use a lighter gray (#B2B2B2)  
    /// - Separators use an even lighter gray (#DDDADA)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::Styles;
    ///
    /// let styles = Styles::default();
    /// ```
    fn default() -> Self {
        let key_style = Style::new().foreground(Color::from("#909090"));
        let desc_style = Style::new().foreground(Color::from("#B2B2B2"));
        let sep_style = Style::new().foreground(Color::from("#DDDADA"));

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
/// based on the `show_all` toggle.
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use bubbletea_widgets::help::{Model, KeyMap};
/// use bubbletea_widgets::key;
///
/// // Create a new help model
/// let help = Model::new().with_width(80);
///
/// // Implement KeyMap for your application
/// struct AppKeyMap;
/// impl KeyMap for AppKeyMap {
///     fn short_help(&self) -> Vec<&key::Binding> {
///         vec![] // Your key bindings
///     }
///     fn full_help(&self) -> Vec<Vec<&key::Binding>> {
///         vec![vec![]] // Your grouped key bindings
///     }
/// }
///
/// let keymap = AppKeyMap;
/// let help_text = help.view(&keymap);
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
    /// conventional constructor-style API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    ///
    /// let help = Model::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum width of the help view.
    ///
    /// When a width is set, the help view will truncate content that exceeds
    /// this limit, showing an ellipsis to indicate truncation.
    ///
    /// # Arguments
    ///
    /// * `width` - Maximum width in characters. Use 0 for no limit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    ///
    /// let help = Model::new().with_width(80);
    /// assert_eq!(help.width, 80);
    /// ```
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Renders the help view based on the current model state.
    ///
    /// This is the main rendering function that switches between short and full
    /// help views based on the `show_all` flag.
    ///
    /// # Arguments
    ///
    /// * `keymap` - An object implementing the `KeyMap` trait that provides
    ///   the key bindings to display.
    ///
    /// # Returns
    ///
    /// A formatted string ready for display in the terminal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::{Model, KeyMap};
    /// use bubbletea_widgets::key;
    ///
    /// struct MyKeyMap;
    /// impl KeyMap for MyKeyMap {
    ///     fn short_help(&self) -> Vec<&key::Binding> { vec![] }
    ///     fn full_help(&self) -> Vec<Vec<&key::Binding>> { vec![] }
    /// }
    ///
    /// let help = Model::new();
    /// let keymap = MyKeyMap;
    /// let rendered = help.view(&keymap);
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
    /// the configured separator. If the content exceeds the specified width,
    /// it will be truncated with an ellipsis.
    ///
    /// # Arguments
    ///
    /// * `bindings` - A vector of key bindings to display.
    ///
    /// # Returns
    ///
    /// A single-line string containing the formatted help text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    /// use bubbletea_widgets::key;
    ///
    /// let help = Model::new();
    /// let bindings = vec![]; // Your key bindings
    /// let short_help = help.short_help_view(bindings);
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
    /// within each column.
    ///
    /// # Arguments
    ///
    /// * `groups` - A vector of key binding groups, where each group becomes
    ///   a column in the output.
    ///
    /// # Returns
    ///
    /// A multi-line string containing the formatted help text with proper
    /// column alignment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::help::Model;
    /// use bubbletea_widgets::key;
    ///
    /// let help = Model::new();
    /// let groups = vec![vec![]]; // Your grouped key bindings
    /// let full_help = help.full_help_view(groups);
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

        for (i, group) in groups.iter().enumerate() {
            if group.is_empty() {
                continue;
            }

            let sep = if i > 0 { &separator } else { "" };

            let keys: Vec<String> = group
                .iter()
                .filter(|b| b.enabled())
                .map(|b| b.help().key.clone())
                .collect();
            let descs: Vec<String> = group
                .iter()
                .filter(|b| b.enabled())
                .map(|b| b.help().desc.clone())
                .collect();

            if keys.is_empty() {
                continue;
            }

            let key_column = self
                .styles
                .full_key
                .clone()
                .inline(true)
                .render(&keys.join("\n"));
            let desc_column = self
                .styles
                .full_desc
                .clone()
                .inline(true)
                .render(&descs.join("\n"));

            let col_str =
                lipgloss::join_horizontal(lipgloss::TOP, &[sep, &key_column, " ", &desc_column]);

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

        lipgloss::join_horizontal(
            lipgloss::TOP,
            &columns.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        )
    }

    /// Determines if an item can be added to the view without exceeding the width limit.
    ///
    /// This helper function checks width constraints and returns appropriate truncation
    /// indicators when content would exceed the configured width.
    ///
    /// # Arguments
    ///
    /// * `total_width` - Current accumulated width of content
    /// * `item_width` - Width of the item being considered for addition
    ///
    /// # Returns
    ///
    /// * `None` - Item can be added without exceeding width
    /// * `Some(String)` - Item cannot be added; string contains ellipsis if it fits,
    ///   or empty string if even ellipsis won't fit
    ///
    /// # Panics
    ///
    /// This function does not panic under normal circumstances.
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
}
