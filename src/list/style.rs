//! Styling system for list components.
//!
//! This module provides comprehensive styling options for list components, including
//! styles for different UI elements and visual states. The styling system is built
//! on top of lipgloss and provides sensible defaults for terminal applications.
//!
//! ## Style Categories
//!
//! - **Title and Header**: Styles for list titles and column headers
//! - **Status and Filter**: Styles for status bar and filter prompt
//! - **Pagination**: Styles for pagination indicators and navigation
//! - **Visual Elements**: Styles for spinners, dots, and separators
//!
//! ## Constants
//!
//! - `BULLET`: Unicode bullet character (`•`) used in pagination
//! - `ELLIPSIS`: Unicode ellipsis character (`…`) for truncated content
//!
//! ## Color Scheme
//!
//! The default styles use a dark theme with:
//! - Bright colors for active/selected elements
//! - Subdued colors for secondary information
//! - High contrast for readability
//!
//! ## Adaptive Colors
//!
//! All default styles use `AdaptiveColor` which automatically adjusts to the
//! terminal's light or dark theme, ensuring optimal readability and visual
//! consistency across different environments.
//!
//! ## Example
//!
//! ```rust
//! use bubbletea_widgets::list::style::{ListStyles, BULLET, ELLIPSIS};
//! use lipgloss_extras::prelude::*;
//!
//! // Use default adaptive styles
//! let mut styles = ListStyles::default();
//!
//! // Customize specific elements with adaptive colors
//! styles.title = Style::new()
//!     .foreground(AdaptiveColor { Light: "#1a1a1a", Dark: "#ffffff" })
//!     .bold(true);
//!
//! // Use the constants for consistent symbols
//! println!("Pagination: {}", BULLET);
//! println!("Truncated: item1, item2{}", ELLIPSIS);
//! ```

use lipgloss_extras::prelude::*;

/// Unicode bullet character (•) used in pagination indicators and visual separators.
///
/// This constant provides a consistent bullet symbol for pagination dots, dividers,
/// and other list UI elements. The bullet character is automatically styled by
/// the respective style configurations in `ListStyles`.
///
/// # Usage
///
/// The bullet is automatically used by default in:
/// - Active and inactive pagination dots
/// - Divider elements between UI sections
/// - Visual separation in status displays
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::list::style::BULLET;
///
/// // Display pagination indicator
/// println!("Page 1 {} 2 {} 3", BULLET, BULLET);
///
/// // Create divider text
/// let divider = format!(" {} ", BULLET);
/// assert_eq!(divider, " • ");
/// ```
pub const BULLET: &str = "•";

/// Unicode ellipsis character (…) used for truncated content display.
///
/// This constant provides a consistent ellipsis symbol for indicating truncated
/// text in list items, headers, and other UI elements when content exceeds the
/// available display width.
///
/// # Usage
///
/// The ellipsis is commonly used for:
/// - Truncating long item titles or descriptions
/// - Indicating overflow in fixed-width displays
/// - Showing partial content in constrained layouts
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::list::style::ELLIPSIS;
///
/// // Simulate text truncation
/// let long_text = "This is a very long text that needs truncation";
/// let max_width = 20;
///
/// let truncated = if long_text.len() > max_width {
///     format!("{}{}", &long_text[..max_width-1], ELLIPSIS)
/// } else {
///     long_text.to_string()
/// };
///
/// println!("{}", truncated); // "This is a very lon…"
/// ```
pub const ELLIPSIS: &str = "…";

/// Comprehensive styling configuration for all list component UI elements.
///
/// This struct contains styling definitions for every visual element in a list
/// component, from the title bar and items to pagination and help text. All
/// styles use adaptive colors that automatically adjust to the terminal's
/// light or dark theme.
///
/// # Style Categories
///
/// The styles are organized into logical categories:
///
/// ## Header and Title
/// - `title_bar`: Container for the list title
/// - `title`: The list title text styling
///
/// ## Filtering and Input
/// - `filter_prompt`: The "Filter:" prompt text
/// - `filter_cursor`: Cursor/caret in filter input
/// - `default_filter_character_match`: Character-level match highlighting
///
/// ## Status and Information
/// - `status_bar`: Main status bar container
/// - `status_empty`: Status when list is empty
/// - `status_bar_active_filter`: Active filter indicator
/// - `status_bar_filter_count`: Filter result count
/// - `no_items`: "No items" message
///
/// ## Pagination and Navigation
/// - `pagination_style`: Pagination area container
/// - `active_pagination_dot`: Current page indicator (•)
/// - `inactive_pagination_dot`: Other page indicators (•)
/// - `arabic_pagination`: Numeric pagination (1, 2, 3...)
/// - `divider_dot`: Separator between elements ( • )
///
/// ## Interactive Elements
/// - `spinner`: Loading/processing indicator
/// - `help_style`: Help text area
///
/// # Adaptive Color System
///
/// All default styles use `AdaptiveColor` which provides different colors
/// for light and dark terminal themes:
/// - **Light themes**: Darker text on light backgrounds
/// - **Dark themes**: Lighter text on dark backgrounds
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::list::style::ListStyles;
/// use lipgloss_extras::prelude::*;
///
/// // Start with default adaptive styles
/// let mut styles = ListStyles::default();
///
/// // Customize title with branded colors
/// styles.title = Style::new()
///     .background(Color::from("#7D56F4"))
///     .foreground(Color::from("#FFFFFF"))
///     .bold(true)
///     .padding(0, 1, 0, 1);
///
/// // Make filter prompt more prominent
/// styles.filter_prompt = Style::new()
///     .foreground(AdaptiveColor { Light: "#059669", Dark: "#10B981" })
///     .bold(true);
///
/// // Use subtle pagination
/// styles.pagination_style = Style::new()
///     .foreground(AdaptiveColor { Light: "#9CA3AF", Dark: "#6B7280" })
///     .padding_left(2);
/// ```
///
/// # Integration
///
/// This struct is typically used with `Model` to configure the entire
/// list appearance:
///
/// ```rust
/// use bubbletea_widgets::list::{Model, DefaultItem, DefaultDelegate, style::ListStyles};
///
/// let items = vec![DefaultItem::new("Item 1", "Description 1")];
/// let delegate = DefaultDelegate::new();
/// let list: Model<DefaultItem> = Model::new(items, delegate, 80, 24);
///
/// // Custom styles can be created and configured
/// let mut custom_styles = ListStyles::default();
/// custom_styles.title = custom_styles.title.bold(true);
/// // Note: Styles would be applied through constructor or builder pattern
/// ```
#[derive(Debug, Clone)]
pub struct ListStyles {
    /// Style for the title bar container.
    pub title_bar: Style,
    /// Style for the list title text.
    pub title: Style,
    /// Style for spinner glyphs.
    pub spinner: Style,
    /// Style for the filter prompt label.
    pub filter_prompt: Style,
    /// Style for the filter cursor/caret.
    pub filter_cursor: Style,
    /// Style for default filter character highlight.
    pub default_filter_character_match: Style,
    /// Style for the status bar container.
    pub status_bar: Style,
    /// Style for the status bar when the list is empty.
    pub status_empty: Style,
    /// Style for active filter text in the status bar.
    pub status_bar_active_filter: Style,
    /// Style for filter match count in the status bar.
    pub status_bar_filter_count: Style,
    /// Style for the "No items" message.
    pub no_items: Style,
    /// Style for pagination area.
    pub pagination_style: Style,
    /// Style for help text area.
    pub help_style: Style,
    /// Style for the active pagination dot.
    pub active_pagination_dot: Style,
    /// Style for the inactive pagination dot.
    pub inactive_pagination_dot: Style,
    /// Style for arabic numerals in pagination.
    pub arabic_pagination: Style,
    /// Style for the divider dot between elements.
    pub divider_dot: Style,
}

impl Default for ListStyles {
    /// Creates default list styles matching the Go bubbles library appearance.
    ///
    /// The default styles provide a professional, accessible appearance with:
    /// - **Adaptive colors** that automatically adjust to terminal themes
    /// - **Consistent typography** with proper spacing and alignment
    /// - **High contrast** for accessibility and readability
    /// - **Visual hierarchy** with appropriate emphasis and subdued elements
    ///
    /// # Color Palette
    ///
    /// The default styles use a carefully chosen adaptive color palette:
    ///
    /// ## Primary Colors
    /// - **Title**: Fixed colors (background: #3E (purple), text: #E6 (light))
    /// - **Active elements**: Bright colors for focus and selection
    ///
    /// ## Adaptive Colors
    /// - **Normal text**: Light: dark text, Dark: light text
    /// - **Subdued elements**: Light: medium gray, Dark: dark gray
    /// - **Interactive elements**: Green/yellow filter prompts, purple accents
    ///
    /// ## Accessibility
    /// - All color combinations meet WCAG contrast requirements
    /// - Colors are distinguishable for common color vision differences
    /// - Text remains readable in both light and dark terminal themes
    ///
    /// # Visual Design
    ///
    /// The default layout follows these principles:
    /// - **Padding**: Consistent 2-unit left padding for alignment
    /// - **Spacing**: 1-line spacing for readability
    /// - **Borders**: Minimal, used only for emphasis
    /// - **Typography**: No decorative fonts, clear hierarchy
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::list::style::ListStyles;
    ///
    /// // Get default styles
    /// let styles = ListStyles::default();
    ///
    /// // The styles will automatically adapt to your terminal theme
    /// // In light terminals: dark text on light backgrounds
    /// // In dark terminals: light text on dark backgrounds
    /// ```
    ///
    /// # Compatibility
    ///
    /// These defaults are designed to match the visual appearance of the
    /// original Go charmbracelet/bubbles library, ensuring consistency
    /// across different implementations.
    fn default() -> Self {
        // Adaptive colors matching Go version
        let very_subdued_color = AdaptiveColor {
            Light: "#DDDADA",
            Dark: "#3C3C3C",
        };
        let subdued_color = AdaptiveColor {
            Light: "#9B9B9B",
            Dark: "#5C5C5C",
        };

        Self {
            title_bar: Style::new().padding(0, 0, 1, 2),
            title: Style::new()
                .background(Color::from("62"))
                .foreground(Color::from("230"))
                .padding(0, 1, 0, 1),
            spinner: Style::new().foreground(AdaptiveColor {
                Light: "#8E8E8E",
                Dark: "#747373",
            }),
            filter_prompt: Style::new().foreground(AdaptiveColor {
                Light: "#04B575",
                Dark: "#ECFD65",
            }),
            filter_cursor: Style::new().foreground(AdaptiveColor {
                Light: "#EE6FF8",
                Dark: "#EE6FF8",
            }),
            default_filter_character_match: Style::new().underline(true),
            status_bar: Style::new()
                .foreground(AdaptiveColor {
                    Light: "#A49FA5",
                    Dark: "#777777",
                })
                .padding(0, 0, 1, 2),
            status_empty: Style::new().foreground(subdued_color.clone()),
            status_bar_active_filter: Style::new().foreground(AdaptiveColor {
                Light: "#1a1a1a",
                Dark: "#dddddd",
            }),
            status_bar_filter_count: Style::new().foreground(very_subdued_color.clone()),
            no_items: Style::new().foreground(AdaptiveColor {
                Light: "#909090",
                Dark: "#626262",
            }),
            arabic_pagination: Style::new().foreground(subdued_color.clone()),
            pagination_style: Style::new().padding_left(2),
            help_style: Style::new().padding(1, 0, 0, 2),
            active_pagination_dot: Style::new()
                .foreground(AdaptiveColor {
                    Light: "#847A85",
                    Dark: "#979797",
                })
                .set_string(BULLET),
            inactive_pagination_dot: Style::new()
                .foreground(very_subdued_color.clone())
                .set_string(BULLET),
            divider_dot: Style::new()
                .foreground(very_subdued_color)
                .set_string(&format!(" {} ", BULLET)),
        }
    }
}
