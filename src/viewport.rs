//! Scrollable viewport component for displaying large content in terminal applications.
//!
//! This module provides a sophisticated viewport component that enables smooth scrolling
//! through content that exceeds the available display area. It supports both vertical
//! and horizontal scrolling, efficient rendering of large datasets, and comprehensive
//! keyboard navigation with customizable key bindings.
//!
//! # Core Features
//!
//! - **Bidirectional Scrolling**: Smooth vertical and horizontal content navigation
//! - **Efficient Rendering**: Only visible content is processed for optimal performance
//! - **Vim-Style Navigation**: Familiar keyboard shortcuts with arrow key alternatives
//! - **Content Management**: Support for both string and line-based content
//! - **Styling Integration**: Full lipgloss styling support with frame calculations
//! - **Mouse Support**: Configurable mouse wheel scrolling (when available)
//! - **Position Tracking**: Precise scroll percentage and boundary detection
//!
//! # Quick Start
//!
//! ```rust
//! use bubbletea_widgets::viewport::{new, Model};
//!
//! // Create a viewport with specific dimensions
//! let mut viewport = new(80, 24);
//!
//! // Set content to display
//! viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5");
//!
//! // Navigate through content
//! viewport.scroll_down(1);     // Scroll down one line
//! viewport.page_down();        // Scroll down one page
//! viewport.goto_bottom();      // Jump to end
//!
//! // Check current state
//! let visible = viewport.visible_lines();
//! let progress = viewport.scroll_percent();
//! ```
//!
//! # Integration with Bubble Tea
//!
//! ```rust
//! use bubbletea_widgets::viewport::{Model as ViewportModel, ViewportKeyMap};
//! use bubbletea_rs::{Model as BubbleTeaModel, Cmd, Msg};
//! use lipgloss_extras::prelude::*;
//!
//! struct DocumentViewer {
//!     viewport: ViewportModel,
//!     content: String,
//! }
//!
//! impl BubbleTeaModel for DocumentViewer {
//!     fn init() -> (Self, Option<Cmd>) {
//!         let mut viewer = DocumentViewer {
//!             viewport: ViewportModel::new(80, 20),
//!             content: "Large document content...".to_string(),
//!         };
//!         viewer.viewport.set_content(&viewer.content);
//!         (viewer, None)
//!     }
//!
//!     fn update(&mut self, msg: Msg) -> Option<Cmd> {
//!         // Forward navigation messages to viewport
//!         self.viewport.update(msg)
//!     }
//!
//!     fn view(&self) -> String {
//!         format!(
//!             "Document Viewer\n\n{}\n\nScroll: {:.1}%",
//!             self.viewport.view(),
//!             self.viewport.scroll_percent() * 100.0
//!         )
//!     }
//! }
//! ```
//!
//! # Advanced Usage
//!
//! ```rust
//! use bubbletea_widgets::viewport::{Model, ViewportKeyMap};
//! use lipgloss_extras::prelude::*;
//!
//! // Create viewport with custom styling
//! let mut viewport = Model::new(60, 15)
//!     .with_style(
//!         Style::new()
//!             .border_style(lipgloss::normal_border())
//!             .border_foreground(Color::from("#874BFD"))
//!             .padding(1, 2, 1, 2)
//!     );
//!
//! // Content from multiple sources
//! let lines: Vec<String> = vec![
//!     "Header information".to_string(),
//!     "Content line 1".to_string(),
//!     "Content line 2".to_string(),
//! ];
//! viewport.set_content_lines(lines);
//!
//! // Configure horizontal scrolling
//! viewport.set_horizontal_step(5); // Scroll 5 columns at a time
//! viewport.scroll_right();         // Move right
//! ```
//!
//! # Navigation Controls
//!
//! | Keys | Action | Description |
//! |------|--------| ----------- |
//! | `↑`, `k` | Line Up | Scroll up one line |
//! | `↓`, `j` | Line Down | Scroll down one line |
//! | `←`, `h` | Left | Scroll left horizontally |
//! | `→`, `l` | Right | Scroll right horizontally |
//! | `PgUp`, `b` | Page Up | Scroll up one page |
//! | `PgDn`, `f`, `Space` | Page Down | Scroll down one page |
//! | `u` | Half Page Up | Scroll up half a page |
//! | `d` | Half Page Down | Scroll down half a page |
//!
//! # Performance Optimization
//!
//! The viewport is designed for efficient handling of large content:
//!
//! - Only visible lines are rendered, regardless of total content size
//! - Scrolling operations return affected lines for incremental updates
//! - String operations are optimized for Unicode content
//! - Frame size calculations account for lipgloss styling overhead
//!
//! # Content Types
//!
//! The viewport supports various content formats:
//!
//! - **Plain text**: Simple string content with automatic line splitting
//! - **Pre-formatted lines**: Vector of strings for precise line control
//! - **Unicode content**: Full support for wide characters and emojis
//! - **Styled content**: Integration with lipgloss for rich formatting
//!
//! # State Management
//!
//! Track viewport state with built-in methods:
//!
//! - `at_top()` / `at_bottom()`: Boundary detection
//! - `scroll_percent()`: Vertical scroll progress (0.0 to 1.0)
//! - `horizontal_scroll_percent()`: Horizontal scroll progress
//! - `line_count()`: Total content lines
//! - `visible_lines()`: Currently displayed content

use crate::key::{self, KeyMap as KeyMapTrait};
use bubbletea_rs::{Cmd, KeyMsg, Model as BubbleTeaModel, Msg};
use crossterm::event::KeyCode;
use lipgloss_extras::lipgloss::width as lg_width;
use lipgloss_extras::prelude::*;
use unicode_width::UnicodeWidthChar;

const SPACEBAR: char = ' ';

/// Keyboard binding configuration for viewport navigation.
///
/// This struct defines all key combinations that control viewport scrolling,
/// including line-by-line movement, page scrolling, and horizontal navigation.
/// Each binding supports multiple key combinations and includes help text for
/// documentation generation.
///
/// # Default Key Bindings
///
/// The default configuration provides both traditional navigation keys and
/// Vim-style alternatives for maximum compatibility:
///
/// - **Line Movement**: Arrow keys (`↑↓`) and Vim keys (`kj`)
/// - **Page Movement**: Page Up/Down and Vim keys (`bf`)
/// - **Half Page**: Vim-style `u` (up) and `d` (down)
/// - **Horizontal**: Arrow keys (`←→`) and Vim keys (`hl`)
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::viewport::{ViewportKeyMap, Model};
/// use bubbletea_widgets::key;
/// use crossterm::event::KeyCode;
///
/// // Use default key bindings
/// let mut viewport = Model::new(80, 24);
/// let keymap = viewport.keymap.clone(); // Uses ViewportKeyMap::default()
///
/// // Customize key bindings
/// let mut custom_keymap = ViewportKeyMap::default();
/// custom_keymap.page_down = key::Binding::new(vec![KeyCode::Char('n')])
///     .with_help("n", "next page");
/// custom_keymap.page_up = key::Binding::new(vec![KeyCode::Char('p')])
///     .with_help("p", "previous page");
///
/// viewport.keymap = custom_keymap;
/// ```
///
/// Integration with help system:
/// ```rust
/// use bubbletea_widgets::viewport::ViewportKeyMap;
/// use bubbletea_widgets::key::KeyMap as KeyMapTrait;
///
/// let keymap = ViewportKeyMap::default();
///
/// // Get essential bindings for compact help
/// let short_help = keymap.short_help();
/// assert_eq!(short_help.len(), 4); // up, down, page_up, page_down
///
/// // Get all bindings organized by category
/// let full_help = keymap.full_help();
/// assert_eq!(full_help.len(), 4); // 4 categories of bindings
/// ```
///
/// # Customization Patterns
///
/// Common customization scenarios:
///
/// ```rust
/// use bubbletea_widgets::viewport::ViewportKeyMap;
/// use bubbletea_widgets::key;
/// use crossterm::event::KeyCode;
///
/// let mut keymap = ViewportKeyMap::default();
///
/// // Add additional keys for page navigation
/// keymap.page_down = key::Binding::new(vec![
///     KeyCode::PageDown,
///     KeyCode::Char(' '),    // Space bar (default)
///     KeyCode::Char('f'),    // Vim style (default)
///     KeyCode::Enter,        // Custom addition
/// ]).with_help("space/f/enter", "next page");
///
/// // Game-style WASD navigation
/// keymap.up = key::Binding::new(vec![KeyCode::Char('w')])
///     .with_help("w", "move up");
/// keymap.down = key::Binding::new(vec![KeyCode::Char('s')])
///     .with_help("s", "move down");
/// keymap.left = key::Binding::new(vec![KeyCode::Char('a')])
///     .with_help("a", "move left");
/// keymap.right = key::Binding::new(vec![KeyCode::Char('d')])
///     .with_help("d", "move right");
/// ```
#[derive(Debug, Clone)]
pub struct ViewportKeyMap {
    /// Key binding for scrolling down one full page.
    ///
    /// Default keys: Page Down, Space, `f` (Vim-style "forward")
    pub page_down: key::Binding,
    /// Key binding for scrolling up one full page.
    ///
    /// Default keys: Page Up, `b` (Vim-style "backward")
    pub page_up: key::Binding,
    /// Key binding for scrolling up half a page.
    ///
    /// Default key: `u` (Vim-style "up half page")
    pub half_page_up: key::Binding,
    /// Key binding for scrolling down half a page.
    ///
    /// Default key: `d` (Vim-style "down half page")
    pub half_page_down: key::Binding,
    /// Key binding for scrolling down one line.
    ///
    /// Default keys: Down arrow (`↓`), `j` (Vim-style)
    pub down: key::Binding,
    /// Key binding for scrolling up one line.
    ///
    /// Default keys: Up arrow (`↑`), `k` (Vim-style)
    pub up: key::Binding,
    /// Key binding for horizontal scrolling to the left.
    ///
    /// Default keys: Left arrow (`←`), `h` (Vim-style)
    pub left: key::Binding,
    /// Key binding for horizontal scrolling to the right.
    ///
    /// Default keys: Right arrow (`→`), `l` (Vim-style)
    pub right: key::Binding,
}

impl Default for ViewportKeyMap {
    /// Creates default viewport key bindings with Vim-style alternatives.
    ///
    /// The default configuration provides comprehensive navigation options
    /// that accommodate both traditional arrow key users and Vim enthusiasts.
    /// Each binding includes multiple key combinations for flexibility.
    ///
    /// # Default Key Mappings
    ///
    /// | Binding | Keys | Description |
    /// |---------|------|-------------|
    /// | `page_down` | `PgDn`, `Space`, `f` | Scroll down one page |
    /// | `page_up` | `PgUp`, `b` | Scroll up one page |
    /// | `half_page_down` | `d` | Scroll down half page |
    /// | `half_page_up` | `u` | Scroll up half page |
    /// | `down` | `↓`, `j` | Scroll down one line |
    /// | `up` | `↑`, `k` | Scroll up one line |
    /// | `left` | `←`, `h` | Scroll left horizontally |
    /// | `right` | `→`, `l` | Scroll right horizontally |
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::ViewportKeyMap;
    ///
    /// // Create with default bindings
    /// let keymap = ViewportKeyMap::default();
    ///
    /// // Verify some default key combinations
    /// assert!(!keymap.page_down.keys().is_empty());
    /// assert!(!keymap.up.keys().is_empty());
    /// ```
    ///
    /// # Design Philosophy
    ///
    /// - **Accessibility**: Arrow keys work for all users
    /// - **Efficiency**: Vim keys provide rapid navigation for power users
    /// - **Consistency**: Key choices match common terminal application patterns
    /// - **Discoverability**: Help text explains each binding clearly
    fn default() -> Self {
        Self {
            page_down: key::Binding::new(vec![
                KeyCode::PageDown,
                KeyCode::Char(SPACEBAR),
                KeyCode::Char('f'),
            ])
            .with_help("f/pgdn", "page down"),
            page_up: key::Binding::new(vec![KeyCode::PageUp, KeyCode::Char('b')])
                .with_help("b/pgup", "page up"),
            half_page_up: key::Binding::new(vec!["u", "ctrl+u"]).with_help("u/ctrl+u", "½ page up"),
            half_page_down: key::Binding::new(vec!["d", "ctrl+d"])
                .with_help("d/ctrl+d", "½ page down"),
            up: key::Binding::new(vec![KeyCode::Up, KeyCode::Char('k')]).with_help("↑/k", "up"),
            down: key::Binding::new(vec![KeyCode::Down, KeyCode::Char('j')])
                .with_help("↓/j", "down"),
            left: key::Binding::new(vec![KeyCode::Left, KeyCode::Char('h')])
                .with_help("←/h", "move left"),
            right: key::Binding::new(vec![KeyCode::Right, KeyCode::Char('l')])
                .with_help("→/l", "move right"),
        }
    }
}

impl KeyMapTrait for ViewportKeyMap {
    /// Returns the most essential key bindings for compact help display.
    ///
    /// This method provides a concise list of the most frequently used
    /// navigation keys, suitable for brief help displays or status bars.
    ///
    /// # Returns
    ///
    /// A vector containing bindings for: up, down, page up, page down
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::ViewportKeyMap;
    /// use bubbletea_widgets::key::KeyMap as KeyMapTrait;
    ///
    /// let keymap = ViewportKeyMap::default();
    /// let essential_keys = keymap.short_help();
    ///
    /// assert_eq!(essential_keys.len(), 4);
    /// // Contains: up, down, page_up, page_down
    /// ```
    fn short_help(&self) -> Vec<&key::Binding> {
        vec![&self.up, &self.down, &self.page_up, &self.page_down]
    }

    /// Returns all key bindings organized by navigation category.
    ///
    /// This method groups related navigation keys together for comprehensive
    /// help displays. Each group represents a logical category of movement.
    ///
    /// # Returns
    ///
    /// A vector of binding groups:
    /// 1. **Line navigation**: up, down
    /// 2. **Horizontal navigation**: left, right  
    /// 3. **Page navigation**: page up, page down
    /// 4. **Half-page navigation**: half page up, half page down
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::ViewportKeyMap;
    /// use bubbletea_widgets::key::KeyMap as KeyMapTrait;
    ///
    /// let keymap = ViewportKeyMap::default();
    /// let all_keys = keymap.full_help();
    ///
    /// assert_eq!(all_keys.len(), 4); // 4 categories
    /// assert_eq!(all_keys[0].len(), 2); // Line navigation: up, down
    /// assert_eq!(all_keys[1].len(), 2); // Horizontal: left, right
    /// assert_eq!(all_keys[2].len(), 2); // Page: page_up, page_down
    /// assert_eq!(all_keys[3].len(), 2); // Half-page: half_page_up, half_page_down
    /// ```
    ///
    /// # Help Display Integration
    ///
    /// This organization enables structured help displays:
    /// ```text
    /// Navigation:
    ///   ↑/k, ↓/j     line up, line down
    ///   ←/h, →/l     scroll left, scroll right
    ///   
    ///   b/pgup, f/pgdn/space    page up, page down
    ///   u, d                     half page up, half page down
    /// ```
    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![
            vec![&self.up, &self.down],
            vec![&self.left, &self.right],
            vec![&self.page_up, &self.page_down],
            vec![&self.half_page_up, &self.half_page_down],
        ]
    }
}

/// High-performance scrollable viewport for displaying large content efficiently.
///
/// This struct represents a complete viewport implementation that can handle content
/// larger than the available display area. It provides smooth scrolling in both
/// vertical and horizontal directions, efficient rendering of only visible content,
/// and comprehensive keyboard navigation.
///
/// # Core Features
///
/// - **Efficient Rendering**: Only visible content is processed, enabling smooth performance with large datasets
/// - **Bidirectional Scrolling**: Full support for both vertical and horizontal content navigation
/// - **Content Management**: Flexible content input via strings or line vectors
/// - **Styling Integration**: Full lipgloss styling support with automatic frame calculations
/// - **Position Tracking**: Precise scroll percentages and boundary detection
/// - **Keyboard Navigation**: Comprehensive key bindings with Vim-style alternatives
///
/// # Examples
///
/// Basic viewport setup:
/// ```rust
/// use bubbletea_widgets::viewport::Model;
///
/// // Create a viewport with specific dimensions
/// let mut viewport = Model::new(80, 24);
///
/// // Add content to display
/// let content = "Line 1\nLine 2\nLine 3\nVery long line that extends beyond viewport width\nLine 5";
/// viewport.set_content(content);
///
/// // Navigate through content
/// viewport.scroll_down(2);  // Move down 2 lines
/// viewport.page_down();     // Move down one page
///
/// // Check current state
/// println!("At bottom: {}", viewport.at_bottom());
/// println!("Scroll progress: {:.1}%", viewport.scroll_percent() * 100.0);
/// ```
///
/// Integration with styling:
/// ```rust
/// use bubbletea_widgets::viewport::Model;
/// use lipgloss_extras::prelude::*;
///
/// let viewport = Model::new(60, 20)
///     .with_style(
///         Style::new()
///             .border_style(lipgloss::normal_border())
///             .border_foreground(Color::from("#874BFD"))
///             .padding(1, 2, 1, 2)
///     );
/// ```
///
/// Working with line-based content:
/// ```rust
/// use bubbletea_widgets::viewport::Model;
///
/// let mut viewport = Model::new(50, 15);
///
/// // Set content from individual lines
/// let lines = vec![
///     "Header Line".to_string(),
///     "Content Line 1".to_string(),
///     "Content Line 2".to_string(),
/// ];
/// viewport.set_content_lines(lines);
///
/// // Get currently visible content
/// let visible = viewport.visible_lines();
/// println!("Displaying {} lines", visible.len());
/// ```
///
/// # Performance Characteristics
///
/// - **Memory**: Only stores content lines, not rendered output
/// - **CPU**: Rendering scales with viewport size, not content size
/// - **Scrolling**: Incremental updates return only affected lines
/// - **Unicode**: Proper width calculation for international content
///
/// # Thread Safety
///
/// The Model struct is `Clone` and can be safely used across threads.
/// All internal state is self-contained and doesn't rely on external resources.
///
/// # State Management
///
/// The viewport maintains several key pieces of state:
/// - **Content**: Lines of text stored internally
/// - **Position**: Current scroll offsets for both axes
/// - **Dimensions**: Viewport size and styling frame calculations
/// - **Configuration**: Mouse settings, scroll steps, and key bindings
#[derive(Debug, Clone)]
pub struct Model {
    /// Display width of the viewport in characters.
    ///
    /// This determines how many characters are visible horizontally.
    /// Content wider than this will require horizontal scrolling to view.
    pub width: usize,
    /// Display height of the viewport in lines.
    ///
    /// This determines how many lines of content are visible at once.
    /// Content with more lines will require vertical scrolling to view.
    pub height: usize,
    /// Lipgloss style applied to the viewport content.
    ///
    /// This style affects the entire viewport area and can include borders,
    /// padding, margins, and background colors. Frame sizes are automatically
    /// calculated and subtracted from the available content area.
    pub style: Style,
    /// Whether mouse wheel scrolling is enabled.
    ///
    /// When `true`, mouse wheel events will scroll the viewport content.
    /// Note: Actual mouse wheel support depends on the terminal and
    /// bubbletea-rs mouse event capabilities.
    pub mouse_wheel_enabled: bool,
    /// Number of lines to scroll per mouse wheel event.
    ///
    /// Default is 3 lines per wheel "click", which provides smooth scrolling
    /// without being too sensitive. Adjust based on content density.
    pub mouse_wheel_delta: usize,
    /// Current vertical scroll position (lines from top).
    ///
    /// This value indicates how many lines have been scrolled down from
    /// the beginning of the content. 0 means showing from the first line.
    pub y_offset: usize,
    /// Current horizontal scroll position (characters from left).
    ///
    /// This value indicates how many characters have been scrolled right
    /// from the beginning of each line. 0 means showing from column 0.
    pub x_offset: usize,
    /// Number of characters to scroll horizontally per step.
    ///
    /// Controls the granularity of horizontal scrolling. Smaller values
    /// provide finer control, larger values enable faster navigation.
    pub horizontal_step: usize,
    /// Vertical position of viewport in terminal for performance rendering.
    ///
    /// Used for optimized rendering in some terminal applications.
    /// Generally can be left at default (0) unless implementing
    /// advanced rendering optimizations.
    pub y_position: usize,
    /// Keyboard binding configuration for navigation.
    ///
    /// Defines which keys control scrolling behavior. Can be customized
    /// to match application-specific navigation patterns or user preferences.
    pub keymap: ViewportKeyMap,

    // Internal state
    /// Content lines stored for display.
    ///
    /// Internal storage for the content being displayed. Managed automatically
    /// when content is set via `set_content()` or `set_content_lines()`.
    lines: Vec<String>,
    /// Width of the longest content line in characters.
    ///
    /// Cached value used for horizontal scrolling calculations and
    /// scroll percentage computations. Updated automatically when content changes.
    longest_line_width: usize,
    /// Whether the viewport has been properly initialized.
    ///
    /// Tracks initialization state to ensure proper configuration.
    /// Set automatically during construction and configuration.
    initialized: bool,
}

impl Model {
    /// Creates a new viewport with the specified dimensions.
    ///
    /// This constructor initializes a viewport with the given width and height,
    /// along with sensible defaults for all configuration options. The viewport
    /// starts with no content and is ready to receive text via `set_content()`
    /// or `set_content_lines()`.
    ///
    /// # Arguments
    ///
    /// * `width` - Display width in characters (horizontal viewport size)
    /// * `height` - Display height in lines (vertical viewport size)
    ///
    /// # Returns
    ///
    /// A new `Model` instance with default configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// // Create a standard terminal-sized viewport
    /// let viewport = Model::new(80, 24);
    /// assert_eq!(viewport.width, 80);
    /// assert_eq!(viewport.height, 24);
    /// assert!(viewport.mouse_wheel_enabled);
    /// ```
    ///
    /// Different viewport sizes for various use cases:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// // Compact viewport for sidebar content
    /// let sidebar = Model::new(30, 20);
    ///
    /// // Wide viewport for code display
    /// let code_view = Model::new(120, 40);
    ///
    /// // Small preview viewport
    /// let preview = Model::new(40, 10);
    /// ```
    ///
    /// # Default Configuration
    ///
    /// - **Mouse wheel**: Enabled with 3-line scroll delta
    /// - **Scroll position**: At top-left (0, 0)
    /// - **Horizontal step**: 1 character per scroll
    /// - **Style**: No styling applied
    /// - **Key bindings**: Vim-style with arrow key alternatives
    ///
    /// # Performance
    ///
    /// Viewport creation is very fast as no content processing occurs during
    /// construction. Memory usage scales with content size, not viewport dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        let mut model = Self {
            width,
            height,
            style: Style::new(),
            mouse_wheel_enabled: true,
            mouse_wheel_delta: 3,
            y_offset: 0,
            x_offset: 0,
            horizontal_step: 1,
            y_position: 0,
            keymap: ViewportKeyMap::default(),
            lines: Vec::new(),
            longest_line_width: 0,
            initialized: false,
        };
        model.set_initial_values();
        model
    }

    /// Set initial values for the viewport
    fn set_initial_values(&mut self) {
        self.mouse_wheel_enabled = true;
        self.mouse_wheel_delta = 3;
        self.initialized = true;
    }

    /// Builder method to set viewport dimensions during construction.
    ///
    /// This method allows for fluent construction by updating the viewport
    /// dimensions after creation. Useful when dimensions are computed or
    /// provided by external sources.
    ///
    /// # Arguments
    ///
    /// * `width` - New width in characters
    /// * `height` - New height in lines
    ///
    /// # Returns
    ///
    /// The modified viewport for method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use lipgloss_extras::prelude::*;
    ///
    /// // Fluent construction with dimensions
    /// let viewport = Model::new(40, 10)
    ///     .with_dimensions(80, 24)
    ///     .with_style(Style::new().padding(1, 2, 1, 2));
    ///
    /// assert_eq!(viewport.width, 80);
    /// assert_eq!(viewport.height, 24);
    /// ```
    ///
    /// Dynamic viewport sizing:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// fn create_responsive_viewport(terminal_width: usize, terminal_height: usize) -> Model {
    ///     Model::new(20, 10) // Default size
    ///         .with_dimensions(
    ///             (terminal_width * 80) / 100,  // 80% of terminal width
    ///             (terminal_height * 60) / 100  // 60% of terminal height
    ///         )
    /// }
    /// ```
    pub fn with_dimensions(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Builder method to apply lipgloss styling to the viewport.
    ///
    /// This method sets the visual styling for the entire viewport area.
    /// The style can include borders, padding, margins, colors, and other
    /// lipgloss formatting. Frame sizes are automatically calculated and
    /// subtracted from the content display area.
    ///
    /// # Arguments
    ///
    /// * `style` - Lipgloss style to apply to the viewport
    ///
    /// # Returns
    ///
    /// The styled viewport for method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use lipgloss_extras::prelude::*;
    ///
    /// // Create viewport with border and padding
    /// let viewport = Model::new(60, 20)
    ///     .with_style(
    ///         Style::new()
    ///             .border_style(lipgloss::normal_border())
    ///             .border_foreground(Color::from("#874BFD"))
    ///             .padding(1, 2, 1, 2)
    ///     );
    /// ```
    ///
    /// Themed viewport styling:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use lipgloss_extras::prelude::*;
    ///
    /// // Dark theme viewport
    /// let dark_viewport = Model::new(80, 24)
    ///     .with_style(
    ///         Style::new()
    ///             .background(Color::from("#1a1a1a"))
    ///             .foreground(Color::from("#ffffff"))
    ///             .border_style(lipgloss::normal_border())
    ///             .border_foreground(Color::from("#444444"))
    ///     );
    ///
    /// // Light theme viewport
    /// let light_viewport = Model::new(80, 24)
    ///     .with_style(
    ///         Style::new()
    ///             .background(Color::from("#ffffff"))
    ///             .foreground(Color::from("#000000"))
    ///             .border_style(lipgloss::normal_border())
    ///             .border_foreground(Color::from("#cccccc"))
    ///     );
    /// ```
    ///
    /// # Frame Size Impact
    ///
    /// Styling with borders and padding reduces the available content area:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use lipgloss_extras::prelude::*;
    ///
    /// // 80x24 viewport with 2-character padding
    /// let viewport = Model::new(80, 24)
    ///     .with_style(
    ///         Style::new().padding(1, 2, 1, 2) // top, right, bottom, left
    ///     );
    ///
    /// // Effective content area is now ~76x22 due to padding
    /// ```
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Returns whether the viewport is scrolled to the very top of the content.
    ///
    /// This method checks if the vertical scroll position is at the beginning,
    /// meaning no content is hidden above the current view. Useful for
    /// determining when scroll-up operations should be disabled or when
    /// displaying scroll indicators.
    ///
    /// # Returns
    ///
    /// `true` if at the top (y_offset == 0), `false` if content is scrolled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// let content = (1..=20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
    /// viewport.set_content(&content);
    ///
    /// // Initially at top
    /// assert!(viewport.at_top());
    ///
    /// // After scrolling down
    /// viewport.scroll_down(1);
    /// assert!(!viewport.at_top());
    ///
    /// // After returning to top
    /// viewport.goto_top();
    /// assert!(viewport.at_top());
    /// ```
    ///
    /// UI integration example:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// fn render_scroll_indicator(viewport: &Model) -> String {
    ///     let up_arrow = if viewport.at_top() { " " } else { "↑" };
    ///     let down_arrow = if viewport.at_bottom() { " " } else { "↓" };
    ///     format!("{} Content {} ", up_arrow, down_arrow)
    /// }
    /// ```
    pub fn at_top(&self) -> bool {
        self.y_offset == 0
    }

    /// Returns whether the viewport is scrolled to or past the bottom of the content.
    ///
    /// This method checks if the vertical scroll position has reached the end,
    /// meaning no more content is available below the current view. Useful for
    /// determining when scroll-down operations should be disabled or when
    /// implementing infinite scroll detection.
    ///
    /// # Returns
    ///
    /// `true` if at or past the bottom, `false` if more content is available below
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 3); // Small viewport
    /// viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5");
    ///
    /// // Initially not at bottom (more content below)
    /// assert!(!viewport.at_bottom());
    ///
    /// // Scroll to bottom
    /// viewport.goto_bottom();
    /// assert!(viewport.at_bottom());
    /// ```
    ///
    /// Scroll control logic:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// fn handle_scroll_down(viewport: &mut Model) -> bool {
    ///     if viewport.at_bottom() {
    ///         // Can't scroll further down
    ///         false
    ///     } else {
    ///         viewport.scroll_down(1);
    ///         true
    ///     }
    /// }
    /// ```
    ///
    /// # Difference from `past_bottom()`
    ///
    /// - `at_bottom()`: Returns `true` when at the maximum valid scroll position
    /// - `past_bottom()`: Returns `true` only when scrolled beyond valid content
    pub fn at_bottom(&self) -> bool {
        self.y_offset >= self.max_y_offset()
    }

    /// Returns whether the viewport has been scrolled beyond valid content.
    ///
    /// This method detects an invalid scroll state where the y_offset exceeds
    /// the maximum valid position. This can occur during content changes or
    /// viewport resizing. Generally indicates a need for scroll position correction.
    ///
    /// # Returns
    ///
    /// `true` if scrolled past valid content, `false` if within valid range
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10);
    /// viewport.set_content("Line 1\nLine 2\nLine 3");
    ///
    /// // Normal scroll position
    /// assert!(!viewport.past_bottom());
    ///
    /// // This would typically be prevented by normal scroll methods,
    /// // but could occur during content changes
    /// ```
    ///
    /// Auto-correction usage:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// fn ensure_valid_scroll(viewport: &mut Model) {
    ///     if viewport.past_bottom() {
    ///         viewport.goto_bottom(); // Correct invalid position
    ///     }
    /// }
    /// ```
    ///
    /// # Use Cases
    ///
    /// - Detecting invalid state after content changes
    /// - Validation in custom scroll implementations
    /// - Debug assertion checks
    /// - Auto-correction logic
    pub fn past_bottom(&self) -> bool {
        self.y_offset > self.max_y_offset()
    }

    /// Returns the vertical scroll progress as a percentage from 0.0 to 1.0.
    ///
    /// This method calculates how far through the content the viewport has
    /// scrolled vertically. 0.0 indicates the top, 1.0 indicates the bottom.
    /// Useful for implementing scroll indicators, progress bars, or proportional
    /// navigation controls.
    ///
    /// # Returns
    ///
    /// A float between 0.0 and 1.0 representing scroll progress:
    /// - `0.0`: At the very top of content
    /// - `0.5`: Halfway through content
    /// - `1.0`: At or past the bottom of content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10");
    ///
    /// // At top
    /// assert_eq!(viewport.scroll_percent(), 0.0);
    ///
    /// // Scroll partway down
    /// viewport.scroll_down(2);
    /// let progress = viewport.scroll_percent();
    /// assert!(progress > 0.0 && progress < 1.0);
    ///
    /// // At bottom
    /// viewport.goto_bottom();
    /// assert_eq!(viewport.scroll_percent(), 1.0);
    /// ```
    ///
    /// Progress bar implementation:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// fn render_progress_bar(viewport: &Model, width: usize) -> String {
    ///     let progress = viewport.scroll_percent();
    ///     let filled_chars = (progress * width as f64) as usize;
    ///     let empty_chars = width - filled_chars;
    ///     
    ///     format!(
    ///         "[{}{}] {:.1}%",
    ///         "█".repeat(filled_chars),
    ///         "░".repeat(empty_chars),
    ///         progress * 100.0
    ///     )
    /// }
    /// ```
    ///
    /// # Special Cases
    ///
    /// - If viewport height >= content lines, returns 1.0 (all content visible)
    /// - Result is clamped to [0.0, 1.0] range for safety
    /// - Calculation accounts for viewport height in determining valid scroll range
    pub fn scroll_percent(&self) -> f64 {
        if self.height >= self.lines.len() {
            return 1.0;
        }
        let y = self.y_offset as f64;
        let h = self.height as f64;
        let t = self.lines.len() as f64;
        let v = y / (t - h);
        v.clamp(0.0, 1.0)
    }

    /// Returns the horizontal scroll progress as a percentage from 0.0 to 1.0.
    ///
    /// This method calculates how far through the content width the viewport has
    /// scrolled horizontally. 0.0 indicates the leftmost position, 1.0 indicates
    /// the rightmost position. Useful for implementing horizontal scroll indicators
    /// or proportional navigation controls for wide content.
    ///
    /// # Returns
    ///
    /// A float between 0.0 and 1.0 representing horizontal scroll progress:
    /// - `0.0`: At the leftmost edge of content
    /// - `0.5`: Halfway through the content width
    /// - `1.0`: At or past the rightmost edge of content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(20, 5);
    /// viewport.set_content("Short line\nThis is a very long line that extends beyond viewport width\nAnother line");
    ///
    /// // At left edge
    /// assert_eq!(viewport.horizontal_scroll_percent(), 0.0);
    ///
    /// // Scroll horizontally
    /// // Scroll right 10 times
    /// for _ in 0..10 {
    ///     viewport.scroll_right();
    /// }
    /// let h_progress = viewport.horizontal_scroll_percent();
    /// assert!(h_progress > 0.0 && h_progress <= 1.0);
    ///
    /// // At right edge
    /// // Scroll far to ensure we reach the end
    /// for _ in 0..1000 {
    ///     viewport.scroll_right();
    /// }
    /// assert_eq!(viewport.horizontal_scroll_percent(), 1.0);
    /// ```
    ///
    /// Horizontal progress indicator:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// fn render_horizontal_indicator(viewport: &Model, width: usize) -> String {
    ///     let h_progress = viewport.horizontal_scroll_percent();
    ///     let position = (h_progress * width as f64) as usize;
    ///     
    ///     let mut indicator = vec!['-'; width];
    ///     if position < width {
    ///         indicator[position] = '|';
    ///     }
    ///     indicator.into_iter().collect()
    /// }
    /// ```
    ///
    /// # Special Cases
    ///
    /// - If viewport width >= longest line width, returns 1.0 (all content visible)
    /// - Result is clamped to [0.0, 1.0] range for safety
    /// - Based on the longest line in the content, not current visible lines
    pub fn horizontal_scroll_percent(&self) -> f64 {
        if self.x_offset >= self.longest_line_width.saturating_sub(self.width) {
            return 1.0;
        }
        let y = self.x_offset as f64;
        let h = self.width as f64;
        let t = self.longest_line_width as f64;
        let v = y / (t - h);
        v.clamp(0.0, 1.0)
    }

    /// Sets the viewport's text content from a multi-line string.
    ///
    /// This method processes the provided string by splitting it into individual lines
    /// and storing them internally for display. Line endings are normalized to Unix-style
    /// (`\n`), and the longest line width is calculated for horizontal scrolling support.
    ///
    /// # Arguments
    ///
    /// * `content` - The text content as a multi-line string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10);
    /// viewport.set_content("Line 1\nLine 2\nVery long line that extends beyond viewport width\nLine 4");
    ///
    /// // Content is now available for display
    /// let visible = viewport.visible_lines();
    /// assert!(!visible.is_empty());
    /// ```
    ///
    /// Loading file content:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use std::fs;
    ///
    /// let mut viewport = Model::new(80, 24);
    ///
    /// // Load file content into viewport
    /// let file_content = fs::read_to_string("example.txt").unwrap_or_default();
    /// viewport.set_content(&file_content);
    /// ```
    ///
    /// Dynamic content updates:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(50, 15);
    ///
    /// // Initial content
    /// viewport.set_content("Initial content\nLine 2");
    ///
    /// // Update with new content
    /// let new_content = (1..=100)
    ///     .map(|i| format!("Generated line {}", i))
    ///     .collect::<Vec<_>>()
    ///     .join("\n");
    /// viewport.set_content(&new_content);
    /// ```
    ///
    /// # Behavior
    ///
    /// - **Line Ending Normalization**: Converts `\r\n` to `\n` for consistency
    /// - **Width Calculation**: Automatically computes the longest line for horizontal scrolling
    /// - **Scroll Position**: If the current scroll position becomes invalid, scrolls to bottom
    /// - **Performance**: Content processing occurs immediately; consider using `set_content_lines()` for pre-split content
    ///
    /// # Cross-Platform Compatibility
    ///
    /// Content from Windows systems with `\r\n` line endings is automatically normalized,
    /// ensuring consistent behavior across all platforms.
    pub fn set_content(&mut self, content: &str) {
        let content = content.replace("\r\n", "\n"); // normalize line endings
        self.lines = content.split('\n').map(|s| s.to_string()).collect();
        self.longest_line_width = find_longest_line_width(&self.lines);

        if self.y_offset > self.lines.len().saturating_sub(1) {
            self.goto_bottom();
        }
    }

    /// Sets the viewport content from a pre-split vector of lines.
    ///
    /// This method directly sets the viewport content from an existing vector of
    /// strings, avoiding the string splitting overhead of `set_content()`. Each
    /// string represents one line of content. This is more efficient when content
    /// is already available as individual lines.
    ///
    /// # Arguments
    ///
    /// * `lines` - Vector of strings where each string is a content line
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10);
    ///
    /// let lines = vec![
    ///     "Header Line".to_string(),
    ///     "Content Line 1".to_string(),
    ///     "Content Line 2".to_string(),
    ///     "A very long line that extends beyond the viewport width".to_string(),
    /// ];
    ///
    /// viewport.set_content_lines(lines);
    ///
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible.len(), 4); // All lines fit in viewport height
    /// ```
    ///
    /// Processing structured data:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// #[derive(Debug)]
    /// struct LogEntry {
    ///     timestamp: String,
    ///     level: String,
    ///     message: String,
    /// }
    ///
    /// let mut viewport = Model::new(80, 20);
    /// let log_entries = vec![
    ///     LogEntry { timestamp: "2024-01-01T10:00:00".to_string(), level: "INFO".to_string(), message: "Application started".to_string() },
    ///     LogEntry { timestamp: "2024-01-01T10:01:00".to_string(), level: "ERROR".to_string(), message: "Connection failed".to_string() },
    /// ];
    ///
    /// let formatted_lines: Vec<String> = log_entries
    ///     .iter()
    ///     .map(|entry| format!("[{}] {}: {}", entry.timestamp, entry.level, entry.message))
    ///     .collect();
    ///
    /// viewport.set_content_lines(formatted_lines);
    /// ```
    ///
    /// Reading from various sources:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(60, 15);
    ///
    /// // Use pre-split lines for better performance
    /// let lines: Vec<String> = vec![
    ///     "Line 1".to_string(),
    ///     "Line 2".to_string(),
    ///     "Line 3".to_string(),
    /// ];
    ///
    /// viewport.set_content_lines(lines);
    /// assert_eq!(viewport.line_count(), 3);
    /// ```
    ///
    /// # Performance Benefits
    ///
    /// - **No String Processing**: Avoids the overhead of splitting a large string
    /// - **Memory Efficient**: Directly moves the vector into internal storage
    /// - **Ideal for Streaming**: Perfect for incrementally building content
    /// - **Pre-formatted Content**: Useful when lines are already processed/formatted
    ///
    /// # Behavior
    ///
    /// - **Width Calculation**: Automatically computes the longest line for horizontal scrolling
    /// - **Scroll Position**: If current scroll position becomes invalid, scrolls to bottom
    /// - **Ownership**: Takes ownership of the provided vector
    /// - **No Normalization**: Lines are used as-is without line ending processing
    pub fn set_content_lines(&mut self, lines: Vec<String>) {
        self.lines = lines;
        self.longest_line_width = find_longest_line_width(&self.lines);

        if self.y_offset > self.lines.len().saturating_sub(1) {
            self.goto_bottom();
        }
    }

    /// Returns the lines currently visible in the viewport.
    ///
    /// This method calculates which lines should be displayed based on the current
    /// scroll position, viewport dimensions, and applied styling. It handles both
    /// vertical scrolling (which lines to show) and horizontal scrolling (which
    /// portion of each line to show). The result accounts for frame sizes from
    /// lipgloss styling like borders and padding.
    ///
    /// # Returns
    ///
    /// A vector of strings representing the currently visible content lines.
    /// Each string is horizontally clipped to fit within the viewport width.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(20, 5);
    /// viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5");
    ///
    /// // Get initial visible lines (height 5 minus 2 frame = 3 effective)
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible.len(), 3);
    /// assert_eq!(visible[0], "Line 1");
    /// assert_eq!(visible[1], "Line 2");
    /// assert_eq!(visible[2], "Line 3");
    ///
    /// // After scrolling down
    /// viewport.scroll_down(2);
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 3");
    /// assert_eq!(visible[1], "Line 4");
    /// assert_eq!(visible[2], "Line 5");
    /// ```
    ///
    /// Horizontal scrolling example:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(10, 4); // Narrow viewport (4 height minus 2 frame = 2 effective)
    /// viewport.set_content("Short\nThis is a very long line that gets clipped");
    ///
    /// // Initial view shows left portion
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[1], "This is ");
    ///
    /// // After horizontal scrolling
    /// // Scroll right 5 times
    /// for _ in 0..5 {
    ///     viewport.scroll_right();
    /// }
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[1], "is a ver"); // Shifted right (8 chars max)
    /// ```
    ///
    /// Working with styled viewport:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use lipgloss_extras::prelude::*;
    ///
    /// let mut viewport = Model::new(20, 5)
    ///     .with_style(
    ///         Style::new().padding(1, 2, 1, 2) // Reduces effective size
    ///     );
    ///
    /// viewport.set_content("Line 1\nLine 2\nLine 3");
    /// let visible = viewport.visible_lines();
    ///
    /// // Available content area is reduced by padding
    /// // Each visible line is also clipped to account for horizontal padding
    /// ```
    ///
    /// # Performance Considerations
    ///
    /// - **Efficient Rendering**: Only processes lines within the visible area
    /// - **Frame Calculation**: Style frame sizes are computed once per call
    /// - **Clipping**: Horizontal clipping is applied only when needed
    /// - **Memory**: Returns a new vector; consider caching for frequent calls
    ///
    /// # Integration Patterns
    ///
    /// This method is typically used in the view/render phase:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use lipgloss_extras::prelude::*;
    ///
    /// fn render_viewport_content(viewport: &Model) -> String {
    ///     let visible_lines = viewport.visible_lines();
    ///     
    ///     if visible_lines.is_empty() {
    ///         return "No content to display".to_string();
    ///     }
    ///     
    ///     visible_lines.join("\n")
    /// }
    /// ```
    pub fn visible_lines(&self) -> Vec<String> {
        let frame_height = self.style.get_vertical_frame_size();
        let frame_width = self.style.get_horizontal_frame_size();
        let h = self.height.saturating_sub(frame_height as usize);
        let w = self.width.saturating_sub(frame_width as usize);

        let mut lines = Vec::new();
        if !self.lines.is_empty() {
            let top = self.y_offset;
            let bottom = (self.y_offset + h).min(self.lines.len());
            lines = self.lines[top..bottom].to_vec();
        }

        // Handle horizontal scrolling
        if self.x_offset == 0 && self.longest_line_width <= w || w == 0 {
            return lines;
        }

        let mut cut_lines = Vec::new();
        for line in lines {
            let cut_line = cut_string(&line, self.x_offset, self.x_offset + w);
            cut_lines.push(cut_line);
        }
        cut_lines
    }

    /// Sets the vertical scroll position to a specific line offset.
    ///
    /// This method directly positions the viewport at the specified line offset
    /// from the beginning of the content. The offset is automatically clamped
    /// to ensure it remains within valid bounds (0 to maximum valid offset).
    ///
    /// # Arguments
    ///
    /// * `n` - The line number to scroll to (0-based indexing)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content(&(1..=20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// // Jump to line 10 (0-based, so content shows "Line 11")
    /// viewport.set_y_offset(10);
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 11");
    ///
    /// // Attempt to scroll beyond content (gets clamped)
    /// viewport.set_y_offset(1000);
    /// assert!(viewport.at_bottom());
    /// ```
    ///
    /// Direct positioning for navigation:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(80, 20);
    /// viewport.set_content("Line content...");
    ///
    /// // Jump to 25% through the content
    /// let total_lines = 100; // Assume we know content size
    /// let quarter_position = total_lines / 4;
    /// viewport.set_y_offset(quarter_position);
    /// ```
    ///
    /// # Clamping Behavior
    ///
    /// - Values less than 0: Set to 0 (top of content)
    /// - Values greater than maximum valid offset: Set to maximum (bottom view)
    /// - Maximum offset ensures at least one line is visible when possible
    ///
    /// # Use Cases
    ///
    /// - **Direct Navigation**: Jump to specific locations
    /// - **Proportional Scrolling**: Navigate based on percentages
    /// - **Search Results**: Position at specific line numbers
    /// - **Bookmarks**: Return to saved positions
    pub fn set_y_offset(&mut self, n: usize) {
        self.y_offset = n.min(self.max_y_offset());
    }

    /// Scrolls down by one full page (viewport height).
    ///
    /// This method moves the viewport down by exactly the viewport height,
    /// effectively showing the next "page" of content. This is the standard
    /// page-down operation found in most text viewers and editors.
    ///
    /// # Returns
    ///
    /// A vector of strings representing the newly visible lines that scrolled into view.
    /// Returns an empty vector if already at the bottom or no scrolling occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content(&(1..=20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// // Initially shows lines 1-5
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 1");
    ///
    /// // Page down shows lines 6-10
    /// let new_lines = viewport.page_down();
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 6");
    /// assert!(!new_lines.is_empty());
    /// ```
    ///
    /// Handling bottom boundary:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content("Line 1\nLine 2\nLine 3"); // Only 3 lines
    ///
    /// // At bottom already, page_down returns empty
    /// viewport.goto_bottom();
    /// let result = viewport.page_down();
    /// assert!(result.is_empty());
    /// ```
    ///
    /// # Performance Optimization
    ///
    /// The returned vector contains only the newly visible lines for efficient
    /// rendering updates. Applications can use this for incremental display updates.
    pub fn page_down(&mut self) -> Vec<String> {
        if self.at_bottom() {
            return Vec::new();
        }
        self.scroll_down(self.height)
    }

    /// Scrolls up by one full page (viewport height).
    ///
    /// This method moves the viewport up by exactly the viewport height,
    /// effectively showing the previous "page" of content. This is the standard
    /// page-up operation found in most text viewers and editors.
    ///
    /// # Returns
    ///
    /// A vector of strings representing the newly visible lines that scrolled into view.
    /// Returns an empty vector if already at the top or no scrolling occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content(&(1..=20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// // Scroll to middle, then page up
    /// viewport.set_y_offset(10);
    /// let new_lines = viewport.page_up();
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 6"); // Moved up by 5 lines
    /// ```
    ///
    /// Handling top boundary:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content("Line 1\nLine 2\nLine 3");
    ///
    /// // Already at top, page_up returns empty
    /// let result = viewport.page_up();
    /// assert!(result.is_empty());
    /// assert!(viewport.at_top());
    /// ```
    pub fn page_up(&mut self) -> Vec<String> {
        if self.at_top() {
            return Vec::new();
        }
        self.scroll_up(self.height)
    }

    /// Scrolls down by half the viewport height.
    ///
    /// This method provides a more granular scrolling option than full page scrolling,
    /// moving the viewport down by half its height. This is commonly mapped to
    /// Ctrl+D in Vim-style navigation.
    ///
    /// # Returns
    ///
    /// A vector of strings representing the newly visible lines that scrolled into view.
    /// Returns an empty vector if already at the bottom or no scrolling occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10); // Height of 10
    /// viewport.set_content(&(1..=30).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// // Half page down moves by 5 lines (10/2)
    /// viewport.half_page_down();
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 6"); // Moved down 5 lines
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Gradual Navigation**: More controlled scrolling than full pages
    /// - **Vim Compatibility**: Matches Ctrl+D behavior
    /// - **Reading Flow**: Maintains better context when scrolling through text
    pub fn half_page_down(&mut self) -> Vec<String> {
        if self.at_bottom() {
            return Vec::new();
        }
        self.scroll_down(self.height / 2)
    }

    /// Scrolls up by half the viewport height.
    ///
    /// This method provides a more granular scrolling option than full page scrolling,
    /// moving the viewport up by half its height. This is commonly mapped to
    /// Ctrl+U in Vim-style navigation.
    ///
    /// # Returns
    ///
    /// A vector of strings representing the newly visible lines that scrolled into view.
    /// Returns an empty vector if already at the top or no scrolling occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10); // Height of 10
    /// viewport.set_content(&(1..=30).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// // Move to middle, then half page up
    /// viewport.set_y_offset(15);
    /// viewport.half_page_up();
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 11"); // Moved up 5 lines (15-5+1)
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Gradual Navigation**: More controlled scrolling than full pages
    /// - **Vim Compatibility**: Matches Ctrl+U behavior  
    /// - **Reading Flow**: Maintains better context when scrolling through text
    pub fn half_page_up(&mut self) -> Vec<String> {
        if self.at_top() {
            return Vec::new();
        }
        self.scroll_up(self.height / 2)
    }

    /// Scrolls down by the specified number of lines.
    ///
    /// This is the fundamental vertical scrolling method that moves the viewport
    /// down by the specified number of lines. All other downward scrolling methods
    /// (page_down, half_page_down) ultimately delegate to this method.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of lines to scroll down
    ///
    /// # Returns
    ///
    /// A vector containing the newly visible lines for performance rendering.
    /// Returns empty vector if no scrolling occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content(&(1..=20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// // Scroll down 3 lines
    /// let new_lines = viewport.scroll_down(3);
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 4"); // Now starting from line 4
    /// assert_eq!(new_lines.len(), 3); // 3 new lines scrolled in
    /// ```
    ///
    /// Edge case handling:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content("Line 1\nLine 2");
    ///
    /// // No scrolling at bottom
    /// viewport.goto_bottom();
    /// let result = viewport.scroll_down(5);
    /// assert!(result.is_empty());
    ///
    /// // No scrolling with n=0
    /// viewport.goto_top();
    /// let result = viewport.scroll_down(0);
    /// assert!(result.is_empty());
    /// ```
    ///
    /// # Performance Optimization
    ///
    /// The returned vector contains only the lines that scrolled into view,
    /// enabling efficient incremental rendering in terminal applications.
    /// This avoids re-rendering the entire viewport when only a few lines changed.
    ///
    /// # Boundary Behavior
    ///
    /// - Automatically stops at the bottom of content
    /// - Returns empty vector if already at bottom
    /// - Handles viewport larger than content gracefully
    pub fn scroll_down(&mut self, n: usize) -> Vec<String> {
        if self.at_bottom() || n == 0 || self.lines.is_empty() {
            return Vec::new();
        }

        self.set_y_offset(self.y_offset + n);

        // Gather lines for performance scrolling
        let bottom = (self.y_offset + self.height).min(self.lines.len());
        let top = (self.y_offset + self.height).saturating_sub(n).min(bottom);
        self.lines[top..bottom].to_vec()
    }

    /// Scrolls up by the specified number of lines.
    ///
    /// This is the fundamental vertical scrolling method that moves the viewport
    /// up by the specified number of lines. All other upward scrolling methods
    /// (page_up, half_page_up) ultimately delegate to this method.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of lines to scroll up
    ///
    /// # Returns
    ///
    /// A vector containing the newly visible lines for performance rendering.
    /// Returns empty vector if no scrolling occurred.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content(&(1..=20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// // Start from middle
    /// viewport.set_y_offset(10);
    ///
    /// // Scroll up 3 lines
    /// let new_lines = viewport.scroll_up(3);
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 8"); // Now starting from line 8 (10-3+1)
    /// assert_eq!(new_lines.len(), 3); // 3 new lines scrolled in
    /// ```
    ///
    /// Edge case handling:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content("Line 1\nLine 2");
    ///
    /// // No scrolling at top
    /// let result = viewport.scroll_up(5);
    /// assert!(result.is_empty());
    /// assert!(viewport.at_top());
    ///
    /// // No scrolling with n=0
    /// let result = viewport.scroll_up(0);
    /// assert!(result.is_empty());
    /// ```
    ///
    /// # Performance Optimization
    ///
    /// The returned vector contains only the lines that scrolled into view,
    /// enabling efficient incremental rendering. Applications can update only
    /// the changed portions of the display.
    ///
    /// # Boundary Behavior
    ///
    /// - Automatically stops at the top of content
    /// - Returns empty vector if already at top
    /// - Uses saturating subtraction to prevent underflow
    pub fn scroll_up(&mut self, n: usize) -> Vec<String> {
        if self.at_top() || n == 0 || self.lines.is_empty() {
            return Vec::new();
        }

        self.set_y_offset(self.y_offset.saturating_sub(n));

        // Gather lines for performance scrolling
        let top = self.y_offset;
        let bottom = (self.y_offset + n).min(self.max_y_offset());
        self.lines[top..bottom].to_vec()
    }

    /// Jumps directly to the beginning of the content.
    ///
    /// This method immediately positions the viewport at the very top of the
    /// content, setting the vertical offset to 0. This is equivalent to pressing
    /// the "Home" key in most text viewers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content(&(1..=20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// // Scroll to middle first
    /// viewport.set_y_offset(10);
    /// assert!(!viewport.at_top());
    ///
    /// // Jump to top
    /// viewport.goto_top();
    /// assert!(viewport.at_top());
    ///
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0], "Line 1");
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Navigation shortcuts**: Quick return to document start
    /// - **Reset position**: Return to initial state after scrolling
    /// - **Search results**: Jump to first occurrence
    /// - **Content refresh**: Start from beginning after content changes
    pub fn goto_top(&mut self) {
        self.y_offset = 0;
    }

    /// Jumps directly to the end of the content.
    ///
    /// This method immediately positions the viewport at the bottom of the
    /// content, showing the last possible page. This is equivalent to pressing
    /// the "End" key in most text viewers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 5);
    /// viewport.set_content(&(1..=20).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// // Jump to bottom
    /// viewport.goto_bottom();
    /// assert!(viewport.at_bottom());
    ///
    /// let visible = viewport.visible_lines();
    /// // With 20 lines total and height 5 (minus 2 for frame), bottom shows last 3 lines
    /// assert_eq!(visible[0], "Line 18");
    /// assert_eq!(visible[2], "Line 20");
    /// ```
    ///
    /// Auto-correction after content changes:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10);
    ///
    /// // Set initial content and scroll down
    /// viewport.set_content(&(1..=50).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    /// viewport.set_y_offset(30);
    ///
    /// // Replace with shorter content
    /// viewport.set_content("Line 1\nLine 2\nLine 3");
    /// // goto_bottom() is called automatically to fix invalid offset
    /// assert!(viewport.at_bottom());
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Navigation shortcuts**: Quick jump to document end
    /// - **Log viewing**: Jump to latest entries
    /// - **Content appending**: Position for new content
    /// - **Auto-correction**: Fix invalid positions after content changes
    pub fn goto_bottom(&mut self) {
        self.y_offset = self.max_y_offset();
    }

    /// Sets the horizontal scrolling step size in characters.
    ///
    /// This method configures how many characters the viewport scrolls
    /// horizontally with each left/right scroll operation. The step size
    /// affects both `scroll_left()` and `scroll_right()` methods.
    ///
    /// # Arguments
    ///
    /// * `step` - Number of characters to scroll per horizontal step
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(20, 5);
    /// viewport.set_content("This is a very long line that extends far beyond the viewport width");
    ///
    /// // Default step is 1 character
    /// viewport.scroll_right();
    /// assert_eq!(viewport.x_offset, 1);
    ///
    /// // Set larger step for faster scrolling
    /// viewport.set_horizontal_step(5);
    /// viewport.scroll_right();
    /// assert_eq!(viewport.x_offset, 6); // 1 + 5
    /// ```
    ///
    /// Different step sizes for different content types:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10);
    ///
    /// // Fine scrolling for precise text viewing
    /// viewport.set_horizontal_step(1);
    ///
    /// // Coarse scrolling for wide data tables
    /// viewport.set_horizontal_step(8); // Tab-like steps
    ///
    /// // Word-based scrolling
    /// viewport.set_horizontal_step(4); // Average word length
    /// ```
    ///
    /// # Use Cases
    ///
    /// - **Fine Control**: Single-character precision (step=1)
    /// - **Tab Columns**: Align with tab stops (step=4 or 8)
    /// - **Word Navigation**: Approximate word-based scrolling
    /// - **Performance**: Larger steps for faster navigation of wide content
    pub fn set_horizontal_step(&mut self, step: usize) {
        self.horizontal_step = step;
    }

    /// Scrolls the viewport left by the configured horizontal step.
    ///
    /// This method moves the horizontal view to the left, revealing content
    /// that was previously hidden on the left side. The scroll amount is
    /// determined by the `horizontal_step` setting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(10, 3);
    /// viewport.set_content("This is a very long line that needs horizontal scrolling");
    ///
    /// // Scroll right first to see the effect of scrolling left
    /// viewport.scroll_right();
    /// viewport.scroll_right();
    /// assert_eq!(viewport.x_offset, 2);
    ///
    /// // Scroll left
    /// viewport.scroll_left();
    /// assert_eq!(viewport.x_offset, 1);
    /// ```
    ///
    /// Boundary handling:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(20, 5);
    /// viewport.set_content("Short content");
    ///
    /// // Already at leftmost position, scroll_left has no effect
    /// assert_eq!(viewport.x_offset, 0);
    /// viewport.scroll_left();
    /// assert_eq!(viewport.x_offset, 0); // Still 0, can't scroll further left
    /// ```
    ///
    /// # Behavior
    ///
    /// - **Boundary Safe**: Uses saturating subtraction to prevent underflow
    /// - **Step-based**: Scrolls by `horizontal_step` amount
    /// - **Immediate**: Takes effect immediately, no animation
    /// - **Absolute Minimum**: Cannot scroll past offset 0 (leftmost position)
    pub fn scroll_left(&mut self) {
        self.x_offset = self.x_offset.saturating_sub(self.horizontal_step);
    }

    /// Scrolls the viewport right by the configured horizontal step.
    ///
    /// This method moves the horizontal view to the right, revealing content
    /// that was previously hidden on the right side. The scroll amount is
    /// determined by the `horizontal_step` setting, and scrolling is limited
    /// by the longest line in the content.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(10, 3);
    /// viewport.set_content("This is a very long line that needs horizontal scrolling");
    ///
    /// // Initial view shows "This is " (width 10 minus 2 for frame = 8 chars)
    /// let visible = viewport.visible_lines();
    /// assert_eq!(visible[0].len(), 8);
    ///
    /// // Scroll right to see more
    /// viewport.scroll_right();
    /// let visible = viewport.visible_lines();
    /// // Now shows "his is a v" (shifted 1 character right)
    /// ```
    ///
    /// Boundary handling:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(20, 5);
    /// viewport.set_content("Short"); // Line shorter than viewport
    ///
    /// // Cannot scroll right when content fits in viewport
    /// viewport.scroll_right();
    /// assert_eq!(viewport.x_offset, 0); // No change
    /// ```
    ///
    /// Multiple step sizes:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(10, 3);
    /// viewport.set_content("Very long line for testing horizontal scrolling behavior");
    ///
    /// // Default single-character scrolling
    /// viewport.scroll_right();
    /// assert_eq!(viewport.x_offset, 1);
    ///
    /// // Change to larger steps
    /// viewport.set_horizontal_step(5);
    /// viewport.scroll_right();
    /// assert_eq!(viewport.x_offset, 6); // 1 + 5
    /// ```
    ///
    /// # Behavior
    ///
    /// - **Content-aware**: Maximum scroll is based on longest line width
    /// - **Viewport-relative**: Considers viewport width in maximum calculation
    /// - **Step-based**: Scrolls by `horizontal_step` amount
    /// - **Clamped**: Cannot scroll past the rightmost useful position
    pub fn scroll_right(&mut self) {
        let max_offset = self.longest_line_width.saturating_sub(self.width);
        self.x_offset = (self.x_offset + self.horizontal_step).min(max_offset);
    }

    /// Get the maximum Y offset
    fn max_y_offset(&self) -> usize {
        let frame_size = self.style.get_vertical_frame_size();
        self.lines
            .len()
            .saturating_sub(self.height.saturating_sub(frame_size as usize))
    }

    /// Returns a reference to the internal content lines.
    ///
    /// This method provides read-only access to all content lines stored in the viewport.
    /// Useful for inspection, searching, or analysis of content without copying.
    ///
    /// # Returns
    ///
    /// A slice containing all content lines as strings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10);
    /// viewport.set_content("Line 1\nLine 2\nLine 3");
    ///
    /// let lines = viewport.lines();
    /// assert_eq!(lines.len(), 3);
    /// assert_eq!(lines[0], "Line 1");
    /// assert_eq!(lines[2], "Line 3");
    /// ```
    ///
    /// Content inspection and search:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10);
    /// viewport.set_content("Line 1\nImportant line\nLine 3");
    ///
    /// // Search for specific content
    /// let lines = viewport.lines();
    /// let important_line = lines.iter().find(|line| line.contains("Important"));
    /// assert!(important_line.is_some());
    /// ```
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Returns the total number of content lines.
    ///
    /// This method returns the count of all content lines, regardless of viewport
    /// dimensions or scroll position. Useful for determining content size,
    /// calculating scroll percentages, or implementing navigation features.
    ///
    /// # Returns
    ///
    /// The total number of lines in the content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10);
    /// viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5");
    ///
    /// assert_eq!(viewport.line_count(), 5);
    ///
    /// // Empty content
    /// viewport.set_content("");
    /// assert_eq!(viewport.line_count(), 1); // Empty string creates one empty line
    /// ```
    ///
    /// Navigation calculations:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let mut viewport = Model::new(40, 10);
    /// viewport.set_content(&(1..=100).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n"));
    ///
    /// let total_lines = viewport.line_count();
    /// let viewport_height = viewport.height;
    ///
    /// // Calculate if scrolling is needed
    /// let needs_scrolling = total_lines > viewport_height;
    /// assert!(needs_scrolling);
    ///
    /// // Calculate maximum number of pages
    /// let max_pages = (total_lines + viewport_height - 1) / viewport_height;
    /// assert_eq!(max_pages, 10); // 100 lines / 10 height = 10 pages
    /// ```
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

impl Default for Model {
    /// Creates a default viewport with standard terminal dimensions.
    ///
    /// The default viewport is sized for typical terminal windows (80x24) and
    /// includes all default configuration options. This is equivalent to calling
    /// `Model::new(80, 24)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    ///
    /// let viewport = Model::default();
    /// assert_eq!(viewport.width, 80);
    /// assert_eq!(viewport.height, 24);
    /// assert!(viewport.mouse_wheel_enabled);
    /// ```
    ///
    /// # Default Configuration
    ///
    /// - **Dimensions**: 80 characters × 24 lines (standard terminal size)
    /// - **Mouse wheel**: Enabled with 3-line scroll delta
    /// - **Scroll position**: At top-left (0, 0)
    /// - **Horizontal step**: 1 character per scroll
    /// - **Style**: No styling applied
    /// - **Key bindings**: Vim-style with arrow key alternatives
    fn default() -> Self {
        Self::new(80, 24)
    }
}

impl BubbleTeaModel for Model {
    /// Initializes a new viewport instance for Bubble Tea applications.
    ///
    /// Creates a default viewport with standard terminal dimensions and no initial commands.
    /// This follows the Bubble Tea initialization pattern where components return their
    /// initial state and any startup commands.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - A default viewport instance (80x24)
    /// - `None` (no initialization commands needed)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use bubbletea_rs::Model as BubbleTeaModel;
    ///
    /// let (viewport, cmd) = Model::init();
    /// assert_eq!(viewport.width, 80);
    /// assert_eq!(viewport.height, 24);
    /// assert!(cmd.is_none());
    /// ```
    fn init() -> (Self, Option<Cmd>) {
        (Self::default(), None)
    }

    /// Processes messages and updates viewport state.
    ///
    /// This method handles keyboard input for viewport navigation, implementing
    /// the standard Bubble Tea update pattern. It processes key messages against
    /// the configured key bindings and updates the viewport scroll position accordingly.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to process (typically keyboard input)
    ///
    /// # Returns
    ///
    /// Always returns `None` as viewport operations don't generate commands
    ///
    /// # Supported Key Bindings
    ///
    /// The default key bindings include:
    /// - **Page navigation**: `f`/`PgDn`/`Space` (page down), `b`/`PgUp` (page up)
    /// - **Half-page navigation**: `d` (half page down), `u` (half page up)
    /// - **Line navigation**: `j`/`↓` (line down), `k`/`↑` (line up)
    /// - **Horizontal navigation**: `l`/`→` (scroll right), `h`/`←` (scroll left)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use bubbletea_rs::{Model as BubbleTeaModel, KeyMsg};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let mut viewport = Model::default();
    /// viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4\nLine 5");
    ///
    /// // Simulate pressing 'j' to scroll down
    /// let key_msg = KeyMsg {
    ///     key: KeyCode::Char('j'),
    ///     modifiers: KeyModifiers::NONE,
    /// };
    ///
    /// let cmd = viewport.update(Box::new(key_msg));
    /// assert!(cmd.is_none());
    /// ```
    ///
    /// # Integration with Bubble Tea
    ///
    /// This method integrates seamlessly with Bubble Tea's message-driven architecture:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use bubbletea_rs::{Model as BubbleTeaModel, Msg};
    ///
    /// struct App {
    ///     viewport: Model,
    /// }
    ///
    /// impl BubbleTeaModel for App {
    /// #   fn init() -> (Self, Option<bubbletea_rs::Cmd>) {
    /// #       (Self { viewport: Model::new(80, 24) }, None)
    /// #   }
    ///     
    ///     fn update(&mut self, msg: Msg) -> Option<bubbletea_rs::Cmd> {
    ///         // Forward messages to viewport
    ///         self.viewport.update(msg);
    ///         None
    ///     }
    /// #
    /// #   fn view(&self) -> String { self.viewport.view() }
    /// }
    /// ```
    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            if self.keymap.page_down.matches(key_msg) {
                self.page_down();
            } else if self.keymap.page_up.matches(key_msg) {
                self.page_up();
            } else if self.keymap.half_page_down.matches(key_msg) {
                self.half_page_down();
            } else if self.keymap.half_page_up.matches(key_msg) {
                self.half_page_up();
            } else if self.keymap.down.matches(key_msg) {
                self.scroll_down(1);
            } else if self.keymap.up.matches(key_msg) {
                self.scroll_up(1);
            } else if self.keymap.left.matches(key_msg) {
                self.scroll_left();
            } else if self.keymap.right.matches(key_msg) {
                self.scroll_right();
            }
        }
        // Mouse wheel basic support if MouseMsg is available in bubbletea-rs
        // Note: bubbletea-rs MouseMsg does not currently expose wheel events in this crate version.
        None
    }

    /// Renders the viewport content as a styled string.
    ///
    /// This method generates the visual representation of the viewport by retrieving
    /// the currently visible lines and applying any configured lipgloss styling.
    /// The output is ready for display in a terminal interface.
    ///
    /// # Returns
    ///
    /// A styled string containing the visible content, ready for terminal output
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use bubbletea_rs::Model as BubbleTeaModel;
    ///
    /// let mut viewport = Model::new(20, 5);
    /// viewport.set_content("Line 1\nLine 2\nLine 3\nLine 4");
    ///
    /// let output = viewport.view();
    /// assert!(output.contains("Line 1"));
    /// assert!(output.contains("Line 2"));
    /// assert!(output.contains("Line 3"));
    /// assert!(!output.contains("Line 4")); // Not visible in 5-line viewport (3 effective)
    /// ```
    ///
    /// With styling applied:
    /// ```rust
    /// use bubbletea_widgets::viewport::Model;
    /// use bubbletea_rs::Model as BubbleTeaModel;
    /// use lipgloss_extras::prelude::*;
    ///
    /// let mut viewport = Model::new(20, 3)
    ///     .with_style(
    ///         Style::new()
    ///             .foreground(Color::from("#FF0000"))
    ///             .background(Color::from("#000000"))
    ///     );
    ///
    /// viewport.set_content("Styled content");
    /// let styled_output = viewport.view();
    /// // Output includes ANSI escape codes for styling
    /// ```
    ///
    /// # Rendering Behavior
    ///
    /// - **Visible Lines Only**: Only renders content within the current viewport
    /// - **Horizontal Clipping**: Content wider than viewport is clipped appropriately  
    /// - **Style Application**: Applied lipgloss styles are rendered into the output
    /// - **Line Joining**: Multiple lines are joined with newline characters
    /// - **Frame Accounting**: Styling frame sizes are automatically considered
    fn view(&self) -> String {
        let visible = self.visible_lines();
        let mut output = String::new();

        for (i, line) in visible.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(line);
        }

        // Apply style if set
        self.style.render(&output)
    }
}

/// Calculates the display width of the longest line in a collection.
///
/// This internal helper function determines the maximum display width among all
/// provided lines, using proper Unicode width calculation via the `lg_width` function.
/// This is essential for horizontal scrolling calculations and determining the
/// maximum horizontal scroll offset.
///
/// # Arguments
///
/// * `lines` - A slice of strings to measure
///
/// # Returns
///
/// The width in characters of the widest line, or 0 if no lines provided
///
/// # Implementation Notes
///
/// - Uses `lg_width()` for proper Unicode width calculation
/// - Handles empty collections gracefully
/// - Accounts for wide characters (CJK, emojis, etc.)
fn find_longest_line_width(lines: &[String]) -> usize {
    lines.iter().map(|line| lg_width(line)).max().unwrap_or(0)
}

/// Extracts a substring based on display width positions for horizontal scrolling.
///
/// This internal helper function cuts a string to show only the portion between
/// specified display width positions. It properly handles Unicode characters with
/// varying display widths, making it essential for horizontal scrolling in the viewport.
///
/// # Arguments
///
/// * `s` - The source string to cut
/// * `start` - The starting display width position (inclusive)
/// * `end` - The ending display width position (exclusive)
///
/// # Returns
///
/// A string containing only the characters within the specified width range
///
/// # Implementation Details
///
/// - **Unicode-aware**: Properly handles wide characters (CJK, emojis)
/// - **Width-based**: Uses display width, not character count
/// - **Boundary safe**: Returns empty string if start is beyond string width
/// - **Performance optimized**: Single pass through characters when possible
///
/// # Examples (Internal Use)
///
/// ```ignore
/// // Wide characters take 2 display columns
/// let result = cut_string("Hello 世界 World", 3, 8);
/// // Shows characters from display column 3 to 7
/// ```
fn cut_string(s: &str, start: usize, end: usize) -> String {
    if start >= lg_width(s) {
        return String::new();
    }

    let chars: Vec<char> = s.chars().collect();
    let mut current_width = 0;
    let mut start_idx = 0;
    let mut end_idx = chars.len();

    // Find start index
    for (i, &ch) in chars.iter().enumerate() {
        if current_width >= start {
            start_idx = i;
            break;
        }
        current_width += ch.width().unwrap_or(0);
    }

    // Find end index
    current_width = 0;
    for (i, &ch) in chars.iter().enumerate() {
        if current_width >= end {
            end_idx = i;
            break;
        }
        current_width += ch.width().unwrap_or(0);
    }

    chars[start_idx..end_idx].iter().collect()
}

/// Creates a new viewport with the specified dimensions.
///
/// This is a convenience function that creates a new viewport instance.
/// It's equivalent to calling `Model::new(width, height)` directly, but
/// provides a more functional style API that some users may prefer.
///
/// # Arguments
///
/// * `width` - Display width in characters
/// * `height` - Display height in lines
///
/// # Returns
///
/// A new viewport `Model` configured with the specified dimensions
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::viewport;
///
/// // Functional style
/// let viewport = viewport::new(80, 24);
///
/// // Equivalent to:
/// let viewport = viewport::Model::new(80, 24);
/// ```
///
/// # Use Cases
///
/// - **Functional Style**: When preferring function calls over constructors
/// - **Import Convenience**: Shorter syntax with `use bubbletea_widgets::viewport::new`
/// - **API Consistency**: Matches the pattern used by other bubbles components
pub fn new(width: usize, height: usize) -> Model {
    Model::new(width, height)
}
