//! Interactive data table component with navigation, selection, and scrolling capabilities.
//!
//! This module provides a comprehensive table implementation for terminal user interfaces,
//! designed for displaying structured data with keyboard navigation and visual selection.
//! It features efficient viewport-based rendering, customizable styling, and integration
//! with the Bubble Tea architecture.
//!
//! # Core Components
//!
//! - **`Model`**: The main table with data, state, and navigation logic
//! - **`Column`**: Column definitions with titles and width constraints
//! - **`Row`**: Row data containing cell values
//! - **`Styles`**: Visual styling for headers, cells, and selection states
//! - **`TableKeyMap`**: Configurable keyboard bindings for navigation
//!
//! # Key Features
//!
//! - **Vim-Style Navigation**: Familiar keyboard shortcuts for power users
//! - **Viewport Scrolling**: Efficient rendering of large datasets
//! - **Selection Highlighting**: Visual feedback for the current row
//! - **Responsive Layout**: Automatic column width and table sizing
//! - **Help Integration**: Built-in documentation of key bindings
//! - **Customizable Styling**: Full control over appearance and colors
//!
//! # Navigation Controls
//!
//! | Keys | Action | Description |
//! |------|--------| ----------- |
//! | `↑`, `k` | Row Up | Move selection up one row |
//! | `↓`, `j` | Row Down | Move selection down one row |
//! | `PgUp`, `b` | Page Up | Move up one page of rows |
//! | `PgDn`, `f` | Page Down | Move down one page of rows |
//! | `u` | Half Page Up | Move up half a page |
//! | `d` | Half Page Down | Move down half a page |
//! | `Home`, `g` | Go to Start | Jump to first row |
//! | `End`, `G` | Go to End | Jump to last row |
//!
//! # Quick Start
//!
//! ```rust
//! use bubbletea_widgets::table::{Model, Column, Row};
//!
//! // Define table structure
//! let columns = vec![
//!     Column::new("Product", 25),
//!     Column::new("Price", 10),
//!     Column::new("Stock", 8),
//! ];
//!
//! // Add data rows
//! let rows = vec![
//!     Row::new(vec!["MacBook Pro".into(), "$2399".into(), "5".into()]),
//!     Row::new(vec!["iPad Air".into(), "$599".into(), "12".into()]),
//!     Row::new(vec!["AirPods Pro".into(), "$249".into(), "23".into()]),
//! ];
//!
//! // Create and configure table
//! let mut table = Model::new(columns)
//!     .with_rows(rows);
//! table.set_width(50);
//! table.set_height(10);
//! ```
//!
//! # Integration with Bubble Tea
//!
//! ```rust
//! use bubbletea_widgets::table::{Model as TableModel, Column, Row};
//! use bubbletea_rs::{Model as BubbleTeaModel, Cmd, Msg, KeyMsg};
//!
//! struct App {
//!     table: TableModel,
//! }
//!
//! impl BubbleTeaModel for App {
//!     fn init() -> (Self, Option<Cmd>) {
//!         let table = TableModel::new(vec![
//!             Column::new("ID", 8),
//!             Column::new("Name", 20),
//!             Column::new("Status", 12),
//!         ]);
//!         (App { table }, None)
//!     }
//!
//!     fn update(&mut self, msg: Msg) -> Option<Cmd> {
//!         // Forward navigation messages to table
//!         self.table.update(msg)
//!     }
//!
//!     fn view(&self) -> String {
//!         format!("My Data Table:\n\n{}", self.table.view())
//!     }
//! }
//! ```
//!
//! # Styling and Customization
//!
//! ```rust
//! use bubbletea_widgets::table::{Model, Column, Row, Styles};
//! use lipgloss_extras::prelude::*;
//!
//! let mut table = Model::new(vec![Column::new("Data", 20)]);
//!
//! // Customize appearance
//! table.styles = Styles {
//!     header: Style::new()
//!         .bold(true)
//!         .background(Color::from("#1e40af"))
//!         .foreground(Color::from("#ffffff"))
//!         .padding(0, 1, 0, 1),
//!     cell: Style::new()
//!         .padding(0, 1, 0, 1),
//!     selected: Style::new()
//!         .bold(true)
//!         .background(Color::from("#10b981"))
//!         .foreground(Color::from("#ffffff")),
//! };
//! ```
//!
//! # Performance Considerations
//!
//! - Uses viewport rendering for efficient display of large datasets
//! - Only visible rows are rendered, enabling smooth performance with thousands of rows
//! - Column widths should be set appropriately to avoid layout recalculation
//! - Selection changes trigger content rebuilding, but viewport limits render cost

use crate::{
    help,
    key::{self, KeyMap as KeyMapTrait},
    viewport,
};
use bubbletea_rs::{Cmd, KeyMsg, Model as BubbleTeaModel, Msg};
use crossterm::event::KeyCode;
use lipgloss_extras::prelude::*;
use lipgloss_extras::table::Table as LGTable;

/// Represents a table column with its title and display width.
///
/// Columns define the structure of the table by specifying headers and how much
/// horizontal space each column should occupy. The width is used for text wrapping
/// and alignment within the column boundaries.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::Column;
///
/// // Create a column for names with 20 character width
/// let name_col = Column::new("Full Name", 20);
/// assert_eq!(name_col.title, "Full Name");
/// assert_eq!(name_col.width, 20);
/// ```
///
/// Multiple columns for a complete table structure:
/// ```rust
/// use bubbletea_widgets::table::Column;
///
/// let columns = vec![
///     Column::new("ID", 8),           // Short numeric ID
///     Column::new("Description", 35), // Longer text field
///     Column::new("Status", 12),      // Medium status field
/// ];
/// ```
///
/// # Width Guidelines
///
/// - **Numeric columns**: 6-10 characters usually sufficient
/// - **Short text**: 10-15 characters for codes, statuses
/// - **Names/titles**: 20-30 characters for readable display
/// - **Descriptions**: 30+ characters for detailed content
#[derive(Debug, Clone)]
pub struct Column {
    /// The display title for this column header.
    ///
    /// This text will be shown in the table header row and should be
    /// descriptive enough for users to understand the column content.
    pub title: String,
    /// The display width for this column in characters.
    ///
    /// This determines how much horizontal space the column occupies.
    /// Content longer than this width will be wrapped or truncated
    /// depending on the styling configuration.
    pub width: i32,
}

impl Column {
    /// Creates a new column with the specified title and width.
    ///
    /// # Arguments
    ///
    /// * `title` - The column header text (accepts any type convertible to String)
    /// * `width` - The display width in characters (should be positive)
    ///
    /// # Returns
    ///
    /// A new `Column` instance ready for use in table construction
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::Column;
    ///
    /// // Using string literals
    /// let col1 = Column::new("User ID", 10);
    ///
    /// // Using String values
    /// let title = String::from("Email Address");
    /// let col2 = Column::new(title, 30);
    ///
    /// // Using &str references
    /// let header = "Created Date";
    /// let col3 = Column::new(header, 15);
    /// ```
    ///
    /// Creating columns for different data types:
    /// ```rust
    /// use bubbletea_widgets::table::Column;
    ///
    /// let columns = vec![
    ///     Column::new("#", 5),              // Row numbers
    ///     Column::new("Name", 25),          // Person names
    ///     Column::new("Email", 30),         // Email addresses
    ///     Column::new("Joined", 12),        // Dates
    ///     Column::new("Active", 8),         // Boolean status
    /// ];
    /// ```
    pub fn new(title: impl Into<String>, width: i32) -> Self {
        Self {
            title: title.into(),
            width,
        }
    }
}

/// Represents a single row of data in the table.
///
/// Each row contains a vector of cell values (as strings) that correspond to the
/// columns defined in the table. The order of cells should match the order of
/// columns for proper display alignment.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::Row;
///
/// // Create a row with user data
/// let user_row = Row::new(vec![
///     "12345".to_string(),
///     "Alice Johnson".to_string(),
///     "alice@example.com".to_string(),
///     "Active".to_string(),
/// ]);
/// assert_eq!(user_row.cells.len(), 4);
/// ```
///
/// Using the `into()` conversion for convenient syntax:
/// ```rust
/// use bubbletea_widgets::table::Row;
///
/// let product_row = Row::new(vec![
///     "SKU-001".into(),
///     "Wireless Mouse".into(),
///     "$29.99".into(),
///     "In Stock".into(),
/// ]);
/// ```
///
/// # Data Alignment
///
/// Cell data should align with column definitions:
/// ```rust
/// use bubbletea_widgets::table::{Column, Row};
///
/// let columns = vec![
///     Column::new("ID", 8),
///     Column::new("Product", 20),
///     Column::new("Price", 10),
/// ];
///
/// // Row data should match column order
/// let row = Row::new(vec![
///     "PROD-123".into(),  // Goes in ID column
///     "Laptop Stand".into(), // Goes in Product column
///     "$49.99".into(),    // Goes in Price column
/// ]);
/// ```
#[derive(Debug, Clone)]
pub struct Row {
    /// The cell values for this row.
    ///
    /// Each string represents the content of one cell, and the vector
    /// order should correspond to the table's column order. All values
    /// are stored as strings regardless of their logical data type.
    pub cells: Vec<String>,
}

impl Row {
    /// Creates a new table row with the specified cell values.
    ///
    /// # Arguments
    ///
    /// * `cells` - Vector of strings representing the data for each column
    ///
    /// # Returns
    ///
    /// A new `Row` instance containing the provided cell data
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::Row;
    ///
    /// // Create a simple data row
    /// let row = Row::new(vec![
    ///     "001".to_string(),
    ///     "John Doe".to_string(),
    ///     "Engineer".to_string(),
    /// ]);
    /// assert_eq!(row.cells[0], "001");
    /// assert_eq!(row.cells[1], "John Doe");
    /// ```
    ///
    /// Using `.into()` for more concise syntax:
    /// ```rust
    /// use bubbletea_widgets::table::Row;
    ///
    /// let employees = vec![
    ///     Row::new(vec!["E001".into(), "Alice".into(), "Manager".into()]),
    ///     Row::new(vec!["E002".into(), "Bob".into(), "Developer".into()]),
    ///     Row::new(vec!["E003".into(), "Carol".into(), "Designer".into()]),
    /// ];
    /// ```
    ///
    /// # Cell Count Considerations
    ///
    /// While not enforced at construction time, rows should typically have the same
    /// number of cells as there are columns in the table:
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Column, Row};
    ///
    /// let columns = vec![
    ///     Column::new("Name", 15),
    ///     Column::new("Age", 5),
    ///     Column::new("City", 15),
    /// ];
    ///
    /// // Good: 3 cells for 3 columns
    /// let good_row = Row::new(vec!["John".into(), "30".into(), "NYC".into()]);
    ///
    /// // Will work but may display oddly: 2 cells for 3 columns
    /// let short_row = Row::new(vec!["Jane".into(), "25".into()]);
    /// ```
    pub fn new(cells: Vec<String>) -> Self {
        Self { cells }
    }
}

/// Visual styling configuration for table rendering.
///
/// This struct defines the appearance of different table elements including headers,
/// regular cells, and selected rows. Each style can control colors, padding, borders,
/// and text formatting using the lipgloss styling system.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::Styles;
/// use lipgloss_extras::prelude::*;
///
/// // Create custom styling
/// let styles = Styles {
///     header: Style::new()
///         .bold(true)
///         .background(Color::from("#2563eb"))
///         .foreground(Color::from("#ffffff"))
///         .padding(0, 1, 0, 1),
///     cell: Style::new()
///         .padding(0, 1, 0, 1)
///         .foreground(Color::from("#374151")),
///     selected: Style::new()
///         .bold(true)
///         .background(Color::from("#10b981"))
///         .foreground(Color::from("#ffffff")),
/// };
/// ```
///
/// Using styles with a table:
/// ```rust
/// use bubbletea_widgets::table::{Model, Column, Styles};
/// use lipgloss_extras::prelude::*;
///
/// let mut table = Model::new(vec![Column::new("Name", 20)]);
/// table.styles = Styles {
///     header: Style::new().bold(true).background(Color::from("blue")),
///     cell: Style::new().padding(0, 1, 0, 1),
///     selected: Style::new().background(Color::from("green")),
/// };
/// ```
///
/// # Styling Guidelines
///
/// - **Headers**: Usually bold with distinct background colors
/// - **Cells**: Minimal styling with consistent padding for readability
/// - **Selected**: High contrast colors to clearly indicate selection
/// - **Padding**: `padding(top, right, bottom, left)` for consistent spacing
#[derive(Debug, Clone)]
pub struct Styles {
    /// Style for table header cells.
    ///
    /// Applied to the first row containing column titles. Typically uses
    /// bold text and distinct background colors to differentiate from data rows.
    pub header: Style,
    /// Style for regular data cells.
    ///
    /// Applied to all non-selected data rows. Should provide good readability
    /// with appropriate padding and neutral colors.
    pub cell: Style,
    /// Style for the currently selected row.
    ///
    /// Applied to highlight the active selection. Should use high contrast
    /// colors to clearly indicate which row is selected.
    pub selected: Style,
}

impl Default for Styles {
    /// Creates default table styling with reasonable visual defaults.
    ///
    /// The default styles provide:
    /// - **Header**: Bold text with padding for clear column identification
    /// - **Cell**: Simple padding for consistent data alignment
    /// - **Selected**: Bold text with a distinct foreground color (#212 - light purple)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Styles, Model, Column};
    ///
    /// // Using default styles
    /// let table = Model::new(vec![Column::new("Data", 20)]);
    /// // table.styles is automatically set to Styles::default()
    ///
    /// // Explicitly using defaults
    /// let styles = Styles::default();
    /// ```
    ///
    /// # Style Details
    ///
    /// - Header padding: `(0, 1, 0, 1)` adds horizontal spacing
    /// - Cell padding: `(0, 1, 0, 1)` maintains consistent alignment
    /// - Selected color: `"212"` is a light purple that works on most terminals
    fn default() -> Self {
        Self {
            header: Style::new().bold(true).padding(0, 1, 0, 1),
            cell: Style::new().padding(0, 1, 0, 1),
            selected: Style::new().bold(true).foreground(Color::from("212")),
        }
    }
}

/// Keyboard binding configuration for table navigation.
///
/// This struct defines all the key combinations that control table navigation,
/// including row-by-row movement, page scrolling, and jumping to start/end positions.
/// Each binding can accept multiple key combinations and includes help text for documentation.
///
/// # Key Binding Types
///
/// - **Row Navigation**: Single row up/down movement
/// - **Page Navigation**: Full page up/down scrolling  
/// - **Half Page Navigation**: Half page up/down scrolling
/// - **Jump Navigation**: Instant movement to start/end
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::{TableKeyMap, Model, Column};
/// use bubbletea_widgets::key;
/// use crossterm::event::KeyCode;
///
/// // Create table with custom key bindings
/// let mut table = Model::new(vec![Column::new("Data", 20)]);
/// table.keymap.row_up = key::Binding::new(vec![KeyCode::Char('w')])
///     .with_help("w", "move up");
/// table.keymap.row_down = key::Binding::new(vec![KeyCode::Char('s')])
///     .with_help("s", "move down");
/// ```
///
/// Using with help system:
/// ```rust
/// use bubbletea_widgets::table::Model;
/// use bubbletea_widgets::key::KeyMap as KeyMapTrait;
///
/// let table = Model::new(vec![]);
/// let help_bindings = table.keymap.short_help();
/// // Returns the most common navigation keys for display
/// ```
///
/// # Default Bindings
///
/// - **Row Up**: `↑` (Up Arrow), `k` (Vim style)
/// - **Row Down**: `↓` (Down Arrow), `j` (Vim style)
/// - **Page Up**: `PgUp`, `b` (Vim style)
/// - **Page Down**: `PgDn`, `f` (Vim style)
/// - **Half Page Up**: `u` (Vim style)
/// - **Half Page Down**: `d` (Vim style)
/// - **Go to Start**: `Home`, `g` (Vim style)
/// - **Go to End**: `End`, `G` (Vim style)
#[derive(Debug, Clone)]
pub struct TableKeyMap {
    /// Key binding for moving selection up one row.
    ///
    /// Default: Up arrow key (`↑`) and `k` key (Vim-style)
    pub row_up: key::Binding,
    /// Key binding for moving selection down one row.
    ///
    /// Default: Down arrow key (`↓`) and `j` key (Vim-style)
    pub row_down: key::Binding,
    /// Key binding for moving up one full page of rows.
    ///
    /// Default: Page Up key and `b` key (Vim-style)
    pub page_up: key::Binding,
    /// Key binding for moving down one full page of rows.
    ///
    /// Default: Page Down key and `f` key (Vim-style)
    pub page_down: key::Binding,
    /// Key binding for moving up half a page of rows.
    ///
    /// Default: `u` key (Vim-style)
    pub half_page_up: key::Binding,
    /// Key binding for moving down half a page of rows.
    ///
    /// Default: `d` key (Vim-style)
    pub half_page_down: key::Binding,
    /// Key binding for jumping to the first row.
    ///
    /// Default: Home key and `g` key (Vim-style)
    pub go_to_start: key::Binding,
    /// Key binding for jumping to the last row.
    ///
    /// Default: End key and `G` key (Vim-style)
    pub go_to_end: key::Binding,
}

impl Default for TableKeyMap {
    /// Creates default table key bindings with Vim-style navigation.
    ///
    /// The default bindings provide both traditional arrow keys and Vim-style letter keys
    /// for maximum compatibility and user preference accommodation.
    ///
    /// # Default Key Mappings
    ///
    /// | Binding | Keys | Description |
    /// |---------|------|-------------|
    /// | `row_up` | `↑`, `k` | Move selection up one row |
    /// | `row_down` | `↓`, `j` | Move selection down one row |
    /// | `page_up` | `PgUp`, `b` | Move up one page of rows |
    /// | `page_down` | `PgDn`, `f` | Move down one page of rows |
    /// | `half_page_up` | `u` | Move up half a page |
    /// | `half_page_down` | `d` | Move down half a page |
    /// | `go_to_start` | `Home`, `g` | Jump to first row |
    /// | `go_to_end` | `End`, `G` | Jump to last row |
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::TableKeyMap;
    ///
    /// // Using default key bindings
    /// let keymap = TableKeyMap::default();
    ///
    /// // Check if a binding includes specific help text
    /// assert_eq!(keymap.row_up.help().key, "↑/k");
    /// assert_eq!(keymap.row_up.help().desc, "up");
    /// ```
    ///
    /// # Design Philosophy
    ///
    /// - **Vim Compatibility**: Letter keys follow Vim navigation patterns
    /// - **Arrow Key Support**: Traditional navigation for all users
    /// - **Page Navigation**: Efficient movement through large datasets
    /// - **Jump Commands**: Quick access to start/end positions
    fn default() -> Self {
        Self {
            row_up: key::Binding::new(vec![KeyCode::Up, KeyCode::Char('k')]).with_help("↑/k", "up"),
            row_down: key::Binding::new(vec![KeyCode::Down, KeyCode::Char('j')])
                .with_help("↓/j", "down"),
            page_up: key::Binding::new(vec![KeyCode::PageUp, KeyCode::Char('b')])
                .with_help("pgup/b", "page up"),
            page_down: key::Binding::new(vec![KeyCode::PageDown, KeyCode::Char('f')])
                .with_help("pgdn/f", "page down"),
            half_page_up: key::Binding::new(vec![KeyCode::Char('u')]).with_help("u", "½ page up"),
            half_page_down: key::Binding::new(vec![KeyCode::Char('d')])
                .with_help("d", "½ page down"),
            go_to_start: key::Binding::new(vec![KeyCode::Home, KeyCode::Char('g')])
                .with_help("g/home", "go to start"),
            go_to_end: key::Binding::new(vec![KeyCode::End, KeyCode::Char('G')])
                .with_help("G/end", "go to end"),
        }
    }
}

impl KeyMapTrait for TableKeyMap {
    fn short_help(&self) -> Vec<&key::Binding> {
        vec![&self.row_up, &self.row_down, &self.page_up, &self.page_down]
    }
    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![
            vec![&self.row_up, &self.row_down],
            vec![&self.page_up, &self.page_down],
            vec![&self.half_page_up, &self.half_page_down],
            vec![&self.go_to_start, &self.go_to_end],
        ]
    }
}

/// Configuration option for table construction.
///
/// This type enables the flexible option-based constructor pattern used by
/// the Go version. Each option is a function that modifies a table model
/// during construction, allowing for clean, composable configuration.
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::{Model, TableOption, with_columns, with_rows, with_height, Column, Row};
///
/// let table = Model::with_options(vec![
///     with_columns(vec![Column::new("Name", 20)]),
///     with_rows(vec![Row::new(vec!["Alice".into()])]),
///     with_height(15),
/// ]);
/// ```
pub type TableOption = Box<dyn FnOnce(&mut Model) + Send>;

/// Creates an option to set table columns during construction.
///
/// This option sets the column structure for the table, defining headers
/// and column widths. This is typically the first option used when
/// creating a new table.
///
/// # Arguments
///
/// * `cols` - Vector of column definitions
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::{Model, with_columns, Column};
///
/// let table = Model::with_options(vec![
///     with_columns(vec![
///         Column::new("ID", 8),
///         Column::new("Name", 25),
///         Column::new("Status", 12),
///     ]),
/// ]);
/// assert_eq!(table.columns.len(), 3);
/// ```
pub fn with_columns(cols: Vec<Column>) -> TableOption {
    Box::new(move |m: &mut Model| {
        m.columns = cols;
    })
}

/// Creates an option to set table rows during construction.
///
/// This option populates the table with initial data rows. Each row
/// should have the same number of cells as there are columns.
///
/// # Arguments
///
/// * `rows` - Vector of row data
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::{Model, with_rows, Row};
///
/// let table = Model::with_options(vec![
///     with_rows(vec![
///         Row::new(vec!["001".into(), "Alice".into()]),
///         Row::new(vec!["002".into(), "Bob".into()]),
///     ]),
/// ]);
/// assert_eq!(table.rows.len(), 2);
/// ```
pub fn with_rows(rows: Vec<Row>) -> TableOption {
    Box::new(move |m: &mut Model| {
        m.rows = rows;
    })
}

/// Creates an option to set table height during construction.
///
/// This option configures the vertical display space for the table,
/// affecting how many rows are visible and viewport scrolling behavior.
///
/// # Arguments
///
/// * `h` - Height in terminal lines
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::{Model, with_height};
///
/// let table = Model::with_options(vec![
///     with_height(25),
/// ]);
/// assert_eq!(table.height, 25);
/// ```
pub fn with_height(h: i32) -> TableOption {
    Box::new(move |m: &mut Model| {
        m.height = h;
        m.sync_viewport_dimensions();
    })
}

/// Creates an option to set table width during construction.
///
/// This option configures the horizontal display space for the table,
/// affecting column layout and content wrapping behavior.
///
/// # Arguments
///
/// * `w` - Width in terminal columns
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::{Model, with_columns, with_width, Column};
///
/// let table = Model::with_options(vec![
///     with_columns(vec![Column::new("Data", 20)]),
///     with_width(80),
/// ]);
/// assert_eq!(table.width, 80);
/// ```
pub fn with_width(w: i32) -> TableOption {
    Box::new(move |m: &mut Model| {
        m.width = w;
        m.sync_viewport_dimensions();
    })
}

/// Creates an option to set table focus state during construction.
///
/// This option configures whether the table should be focused (and thus
/// respond to keyboard input) when initially created.
///
/// # Arguments
///
/// * `f` - `true` for focused, `false` for unfocused
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::{Model, with_focused};
///
/// let table = Model::with_options(vec![
///     with_focused(false),
/// ]);
/// assert!(!table.focus);
/// ```
pub fn with_focused(f: bool) -> TableOption {
    Box::new(move |m: &mut Model| {
        m.focus = f;
    })
}

/// Creates an option to set table styles during construction.
///
/// This option applies custom styling configuration to the table,
/// controlling the appearance of headers, cells, and selection.
///
/// # Arguments
///
/// * `s` - Styling configuration
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::{Model, with_styles, Styles};
/// use lipgloss_extras::prelude::*;
///
/// let custom_styles = Styles {
///     header: Style::new().bold(true),
///     cell: Style::new().padding(0, 1, 0, 1),
///     selected: Style::new().background(Color::from("green")),
/// };
///
/// let table = Model::with_options(vec![
///     with_styles(custom_styles),
/// ]);
/// ```
pub fn with_styles(s: Styles) -> TableOption {
    Box::new(move |m: &mut Model| {
        m.styles = s;
    })
}

/// Creates an option to set table key map during construction.
///
/// This option applies custom key bindings to the table, allowing
/// applications to override the default navigation keys.
///
/// # Arguments
///
/// * `km` - Key mapping configuration
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::table::{Model, with_key_map, TableKeyMap};
/// use bubbletea_widgets::key;
/// use crossterm::event::KeyCode;
///
/// let mut custom_keymap = TableKeyMap::default();
/// custom_keymap.row_up = key::Binding::new(vec![KeyCode::Char('w')])
///     .with_help("w", "up");
///
/// let table = Model::with_options(vec![
///     with_key_map(custom_keymap),
/// ]);
/// ```
pub fn with_key_map(km: TableKeyMap) -> TableOption {
    Box::new(move |m: &mut Model| {
        m.keymap = km;
    })
}

/// Interactive table model containing data, styling, navigation state,
/// and a viewport for efficient rendering and scrolling.
#[derive(Debug, Clone)]
pub struct Model {
    /// Column definitions controlling headers and widths.
    pub columns: Vec<Column>,
    /// Row data; each row contains one string per column.
    pub rows: Vec<Row>,
    /// Index of the currently selected row (0-based).
    pub selected: usize,
    /// Rendered width of the table in characters.
    pub width: i32,
    /// Rendered height of the table in lines.
    pub height: i32,
    /// Key bindings for navigation and movement.
    pub keymap: TableKeyMap,
    /// Styles used when rendering the table.
    pub styles: Styles,
    /// Whether this table currently has keyboard focus.
    pub focus: bool,
    /// Help model used to render key binding help.
    pub help: help::Model,
    /// Internal viewport that manages scrolling of rendered lines.
    viewport: viewport::Model,
}

impl Model {
    /// Creates a new table with the given `columns` and sensible defaults.
    ///
    /// Defaults: height 20, focused, empty rows, and default styles/keymap.
    pub fn new(columns: Vec<Column>) -> Self {
        let mut s = Self {
            columns,
            rows: Vec::new(),
            selected: 0,
            width: 0,
            height: 20,
            keymap: TableKeyMap::default(),
            styles: Styles::default(),
            focus: true,
            help: help::Model::new(),
            viewport: viewport::Model::new(0, 0),
        };
        // Initialize viewport dimensions
        s.sync_viewport_dimensions();
        s.rebuild_viewport_content();
        s
    }

    /// Creates a new table with configuration options (Go-compatible constructor).
    ///
    /// This constructor provides a flexible, option-based approach to table creation
    /// that matches the Go version's `New(opts...)` pattern. Each option is a
    /// function that configures a specific aspect of the table.
    ///
    /// # Arguments
    ///
    /// * `opts` - Vector of configuration options to apply
    ///
    /// # Returns
    ///
    /// A configured table model with all options applied
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, with_columns, with_rows, with_height, Column, Row};
    ///
    /// // Create a fully configured table
    /// let table = Model::with_options(vec![
    ///     with_columns(vec![
    ///         Column::new("ID", 8),
    ///         Column::new("Name", 20),
    ///         Column::new("Status", 12),
    ///     ]),
    ///     with_rows(vec![
    ///         Row::new(vec!["001".into(), "Alice".into(), "Active".into()]),
    ///         Row::new(vec!["002".into(), "Bob".into(), "Inactive".into()]),
    ///     ]),
    ///     with_height(15),
    /// ]);
    /// ```
    ///
    /// Creating an empty table (equivalent to Go's `New()`):
    /// ```rust
    /// use bubbletea_widgets::table::Model;
    ///
    /// let table = Model::with_options(vec![]);
    /// assert_eq!(table.columns.len(), 0);
    /// assert_eq!(table.rows.len(), 0);
    /// ```
    ///
    /// # Constructor Philosophy
    ///
    /// This pattern provides the same flexibility as the Go version while
    /// maintaining Rust's type safety and ownership semantics. Options are
    /// applied in the order provided, allowing later options to override
    /// earlier ones if they configure the same property.
    pub fn with_options(opts: Vec<TableOption>) -> Self {
        let mut m = Self {
            columns: Vec::new(),
            rows: Vec::new(),
            selected: 0,
            width: 0,
            height: 20,
            keymap: TableKeyMap::default(),
            styles: Styles::default(),
            focus: true,
            help: help::Model::new(),
            viewport: viewport::Model::new(0, 0),
        };

        // Apply all options in order
        for opt in opts {
            opt(&mut m);
        }

        // Initialize viewport after all options are applied
        m.sync_viewport_dimensions();
        m.rebuild_viewport_content();
        m
    }

    /// Sets the table rows on construction and returns `self` for chaining.
    pub fn with_rows(mut self, rows: Vec<Row>) -> Self {
        self.rows = rows;
        self
    }
    /// Sets the table width in characters and rebuilds the viewport content.
    pub fn set_width(&mut self, w: i32) {
        self.width = w;
        self.sync_viewport_dimensions();
        self.rebuild_viewport_content();
    }
    /// Sets the table height in lines and rebuilds the viewport content.
    pub fn set_height(&mut self, h: i32) {
        self.height = h;
        self.sync_viewport_dimensions();
        self.rebuild_viewport_content();
    }
    /// Appends a row to the table and refreshes the rendered content.
    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
        self.rebuild_viewport_content();
    }
    /// Returns a reference to the currently selected row, if any.
    pub fn selected_row(&self) -> Option<&Row> {
        self.rows.get(self.selected)
    }
    /// Moves the selection down by one row.
    pub fn select_next(&mut self) {
        if !self.rows.is_empty() {
            self.selected = (self.selected + 1).min(self.rows.len() - 1);
        }
    }
    /// Moves the selection up by one row.
    pub fn select_prev(&mut self) {
        if !self.rows.is_empty() {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Moves the selection up by the specified number of rows (Go-compatible alias).
    ///
    /// This method provides Go API compatibility by matching the `MoveUp` method
    /// signature and behavior from the original table implementation.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of rows to move up
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column, Row};
    ///
    /// let mut table = Model::new(vec![Column::new("Data", 20)]);
    /// table.rows = vec![
    ///     Row::new(vec!["Row 1".into()]),
    ///     Row::new(vec!["Row 2".into()]),
    ///     Row::new(vec!["Row 3".into()]),
    /// ];
    /// table.selected = 2;
    ///
    /// table.move_up(1);
    /// assert_eq!(table.selected, 1);
    /// ```
    pub fn move_up(&mut self, n: usize) {
        if !self.rows.is_empty() {
            self.selected = self.selected.saturating_sub(n);
        }
    }

    /// Moves the selection down by the specified number of rows (Go-compatible alias).
    ///
    /// This method provides Go API compatibility by matching the `MoveDown` method
    /// signature and behavior from the original table implementation.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of rows to move down
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column, Row};
    ///
    /// let mut table = Model::new(vec![Column::new("Data", 20)]);
    /// table.rows = vec![
    ///     Row::new(vec!["Row 1".into()]),
    ///     Row::new(vec!["Row 2".into()]),
    ///     Row::new(vec!["Row 3".into()]),
    /// ];
    /// table.selected = 0;
    ///
    /// table.move_down(2);
    /// assert_eq!(table.selected, 2);
    /// ```
    pub fn move_down(&mut self, n: usize) {
        if !self.rows.is_empty() {
            self.selected = (self.selected + n).min(self.rows.len() - 1);
        }
    }

    /// Moves the selection to the first row (Go-compatible alias).
    ///
    /// This method provides Go API compatibility by matching the `GotoTop` method
    /// from the original table implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column, Row};
    ///
    /// let mut table = Model::new(vec![Column::new("Data", 20)]);
    /// table.rows = vec![
    ///     Row::new(vec!["Row 1".into()]),
    ///     Row::new(vec!["Row 2".into()]),
    ///     Row::new(vec!["Row 3".into()]),
    /// ];
    /// table.selected = 2;
    ///
    /// table.goto_top();
    /// assert_eq!(table.selected, 0);
    /// ```
    pub fn goto_top(&mut self) {
        self.selected = 0;
    }

    /// Moves the selection to the last row (Go-compatible alias).
    ///
    /// This method provides Go API compatibility by matching the `GotoBottom` method
    /// from the original table implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column, Row};
    ///
    /// let mut table = Model::new(vec![Column::new("Data", 20)]);
    /// table.rows = vec![
    ///     Row::new(vec!["Row 1".into()]),
    ///     Row::new(vec!["Row 2".into()]),
    ///     Row::new(vec!["Row 3".into()]),
    /// ];
    /// table.selected = 0;
    ///
    /// table.goto_bottom();
    /// assert_eq!(table.selected, 2);
    /// ```
    pub fn goto_bottom(&mut self) {
        if !self.rows.is_empty() {
            self.selected = self.rows.len() - 1;
        }
    }

    /// Sets table styles and rebuilds the viewport content.
    ///
    /// This method matches the Go version's `SetStyles` functionality by updating
    /// the table's visual styling and ensuring the viewport content is rebuilt
    /// to reflect the new styles.
    ///
    /// # Arguments
    ///
    /// * `s` - The new styling configuration to apply
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column, Styles};
    /// use lipgloss_extras::prelude::*;
    ///
    /// let mut table = Model::new(vec![Column::new("Data", 20)]);
    ///
    /// let custom_styles = Styles {
    ///     header: Style::new().bold(true).background(Color::from("blue")),
    ///     cell: Style::new().padding(0, 1, 0, 1),
    ///     selected: Style::new().background(Color::from("green")),
    /// };
    ///
    /// table.set_styles(custom_styles);
    /// // Table now uses the new styles and viewport is updated
    /// ```
    pub fn set_styles(&mut self, s: Styles) {
        self.styles = s;
        self.update_viewport();
    }

    /// Updates the viewport content based on current columns, rows, and styling.
    ///
    /// This method matches the Go version's `UpdateViewport` functionality by
    /// rebuilding the rendered table content and ensuring the selected row
    /// remains visible. It should be called after any changes to table
    /// structure, data, or styling.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column, Row};
    ///
    /// let mut table = Model::new(vec![Column::new("Name", 20)]);
    /// table.rows.push(Row::new(vec!["Alice".into()]));
    ///
    /// // After manual changes, update the viewport
    /// table.update_viewport();
    /// ```
    ///
    /// # When to Call
    ///
    /// This method is automatically called by most table methods, but you may
    /// need to call it manually when:
    /// - Directly modifying the `rows` or `columns` fields
    /// - Changing dimensions or styling outside of provided methods
    /// - Ensuring content is current after external modifications
    pub fn update_viewport(&mut self) {
        self.rebuild_viewport_content();
    }

    /// Renders help information for table navigation keys.
    ///
    /// This method matches the Go version's `HelpView` functionality by
    /// generating formatted help text that documents all available key
    /// bindings for table navigation.
    ///
    /// # Returns
    ///
    /// A formatted string containing help information for table navigation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column};
    ///
    /// let table = Model::new(vec![Column::new("Data", 20)]);
    /// let help_text = table.help_view();
    ///
    /// // Contains formatted help showing navigation keys
    /// println!("Table Help:\n{}", help_text);
    /// ```
    ///
    /// # Integration
    ///
    /// This method is typically used to display help information separately
    /// from the main table view:
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column};
    ///
    /// struct App {
    ///     table: Model,
    ///     show_help: bool,
    /// }
    ///
    /// impl App {
    ///     fn view(&self) -> String {
    ///         let mut output = self.table.view();
    ///         if self.show_help {
    ///             output.push_str("\n\n");
    ///             output.push_str(&self.table.help_view());
    ///         }
    ///         output
    ///     }
    /// }
    /// ```
    pub fn help_view(&self) -> String {
        self.help.view(self)
    }

    /// Renders the table as a string.
    pub fn view(&self) -> String {
        // Render table directly to string
        let mut tbl = LGTable::new();
        if self.width > 0 {
            tbl = tbl.width(self.width);
        }

        let headers: Vec<String> = self.columns.iter().map(|c| c.title.clone()).collect();
        tbl = tbl.headers(headers);

        let widths = self.columns.iter().map(|c| c.width).collect::<Vec<_>>();
        let cell_style = self.styles.cell.clone();
        let header_style = self.styles.header.clone();
        let selected_row = self.selected as i32;
        let selected_style = self.styles.selected.clone();
        tbl = tbl.style_func_boxed(Box::new(move |row: i32, col: usize| {
            let mut s = if row == lipgloss_extras::table::HEADER_ROW {
                header_style.clone()
            } else {
                cell_style.clone()
            };
            if let Some(w) = widths.get(col) {
                s = s.width(*w);
            }
            if row >= 0 && row == selected_row {
                s = selected_style.clone().inherit(s);
            }
            s
        }));

        let row_vecs: Vec<Vec<String>> = self.rows.iter().map(|r| r.cells.clone()).collect();
        tbl = tbl.rows(row_vecs);
        tbl.to_string()
    }

    fn rebuild_viewport_content(&mut self) {
        let mut tbl = LGTable::new();
        if self.width > 0 {
            tbl = tbl.width(self.width);
        }
        // Don't set table height; viewport will handle vertical scrolling

        // Headers
        let headers: Vec<String> = self.columns.iter().map(|c| c.title.clone()).collect();
        tbl = tbl.headers(headers);

        // Column widths via style_func
        let widths = self.columns.iter().map(|c| c.width).collect::<Vec<_>>();
        let cell_style = self.styles.cell.clone();
        let header_style = self.styles.header.clone();
        let selected_row = self.selected as i32; // data rows are 0-based in lipgloss-table
        let selected_style = self.styles.selected.clone();
        tbl = tbl.style_func_boxed(Box::new(move |row: i32, col: usize| {
            let mut s = if row == lipgloss_extras::table::HEADER_ROW {
                header_style.clone()
            } else {
                cell_style.clone()
            };
            if let Some(w) = widths.get(col) {
                s = s.width(*w);
            }
            if row >= 0 && row == selected_row {
                s = selected_style.clone().inherit(s);
            }
            s
        }));

        // Rows
        let row_vecs: Vec<Vec<String>> = self.rows.iter().map(|r| r.cells.clone()).collect();
        tbl = tbl.rows(row_vecs);

        let rendered = tbl.to_string();
        let lines: Vec<String> = rendered.split('\n').map(|s| s.to_string()).collect();
        self.viewport.set_content_lines(lines);

        // Ensure selection is visible (header is line 0; rows begin at line 1)
        self.ensure_selected_visible();
    }

    fn ensure_selected_visible(&mut self) {
        let target_line = self.selected.saturating_add(1); // account for header
        let h = (self.height.max(1)) as usize;
        let top = self.viewport.y_offset;
        let bottom = top.saturating_add(h.saturating_sub(1));
        if target_line < top {
            self.viewport.set_y_offset(target_line);
        } else if target_line > bottom {
            let new_top = target_line.saturating_sub(h.saturating_sub(1));
            self.viewport.set_y_offset(new_top);
        }
    }

    fn sync_viewport_dimensions(&mut self) {
        self.viewport.width = self.width.max(0) as usize;
        self.viewport.height = self.height.max(0) as usize;
    }

    /// Gives keyboard focus to the table.
    pub fn focus(&mut self) {
        self.focus = true;
    }
    /// Removes keyboard focus from the table.
    pub fn blur(&mut self) {
        self.focus = false;
    }
}

impl BubbleTeaModel for Model {
    /// Creates a new empty table model for Bubble Tea applications.
    ///
    /// This initialization method creates a table with no columns or data,
    /// suitable for applications that will configure the table structure
    /// after initialization. The table starts focused and ready to receive
    /// keyboard input.
    ///
    /// # Returns
    ///
    /// A tuple containing the new table model and no initial command
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::Model;
    /// use bubbletea_rs::Model as BubbleTeaModel;
    ///
    /// // This is typically called by the Bubble Tea framework
    /// let (mut table, cmd) = Model::init();
    /// assert_eq!(table.columns.len(), 0);
    /// assert_eq!(table.rows.len(), 0);
    /// assert!(cmd.is_none());
    /// ```
    ///
    /// # Note
    ///
    /// Most applications will want to use `Model::new(columns)` directly
    /// instead of this init method, as it allows specifying the table
    /// structure immediately.
    fn init() -> (Self, Option<Cmd>) {
        (Self::new(Vec::new()), None)
    }

    /// Processes messages and updates table state with keyboard navigation.
    ///
    /// This method handles all keyboard navigation for the table, including
    /// row selection, page scrolling, and jumping to start/end positions.
    /// It only processes messages when the table is focused, ensuring proper
    /// behavior in multi-component applications.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to process, typically a `KeyMsg` for keyboard input
    ///
    /// # Returns
    ///
    /// An optional `Cmd` that may need to be executed (currently always `None`)
    ///
    /// # Key Handling
    ///
    /// The following keys are processed based on the table's key map configuration:
    ///
    /// - **Row Navigation**: Up/Down arrows, `k`/`j` keys
    /// - **Page Navigation**: Page Up/Down, `b`/`f` keys  
    /// - **Half Page**: `u`/`d` keys for half-page scrolling
    /// - **Jump Navigation**: Home/End, `g`/`G` keys for start/end
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column};
    /// use bubbletea_rs::{KeyMsg, Model as BubbleTeaModel};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// let mut table = Model::new(vec![Column::new("Data", 20)]);
    ///
    /// // Simulate down arrow key press
    /// let key_msg = Box::new(KeyMsg {
    ///     key: KeyCode::Down,
    ///     modifiers: KeyModifiers::NONE,
    /// });
    /// let cmd = table.update(key_msg);
    /// // Table selection moves down (if there are rows)
    /// ```
    ///
    /// # Focus Handling
    ///
    /// If the table is not focused (`self.focus == false`), this method
    /// returns immediately without processing the message. This allows
    /// multiple components to coexist without interference.
    ///
    /// # Performance Optimization
    ///
    /// After any navigation that changes the selection, the viewport content
    /// is automatically rebuilt to ensure the selected row remains visible
    /// and the display is updated correctly.
    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        if let Some(k) = msg.downcast_ref::<KeyMsg>() {
            if !self.focus {
                return None;
            }
            if self.keymap.row_up.matches(k) {
                self.select_prev();
            } else if self.keymap.row_down.matches(k) {
                self.select_next();
            } else if self.keymap.go_to_start.matches(k) {
                self.selected = 0;
            } else if self.keymap.go_to_end.matches(k) {
                if !self.rows.is_empty() {
                    self.selected = self.rows.len() - 1;
                }
            }
            // Page and half-page moves adjust selection relative to height
            else if self.keymap.page_up.matches(k) {
                self.selected = self.selected.saturating_sub(self.height as usize);
            } else if self.keymap.page_down.matches(k) {
                self.selected =
                    (self.selected + self.height as usize).min(self.rows.len().saturating_sub(1));
            } else if self.keymap.half_page_up.matches(k) {
                self.selected = self
                    .selected
                    .saturating_sub((self.height as usize).max(1) / 2);
            } else if self.keymap.half_page_down.matches(k) {
                self.selected = (self.selected + (self.height as usize).max(1) / 2)
                    .min(self.rows.len().saturating_sub(1));
            }
            // After any movement, ensure visibility without rebuilding content
            self.ensure_selected_visible();
        }
        None
    }

    /// Renders the table for display in a Bubble Tea application.
    ///
    /// This method delegates to the table's own `view()` method to generate
    /// the formatted string representation. It's called by the Bubble Tea
    /// framework during the render cycle.
    ///
    /// # Returns
    ///
    /// A multi-line string containing the formatted table with headers,
    /// data rows, selection highlighting, and applied styling
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column, Row};
    /// use bubbletea_rs::Model as BubbleTeaModel;
    ///
    /// let mut table = Model::new(vec![Column::new("Name", 15)]);
    /// table.add_row(Row::new(vec!["Alice".into()]));
    ///
    /// let output = table.view();
    /// // Contains formatted table ready for terminal display
    /// ```
    ///
    /// # Integration Pattern
    ///
    /// This method is typically called from your application's main `view()` method:
    ///
    /// ```rust
    /// use bubbletea_widgets::table::Model as TableModel;
    /// use bubbletea_rs::Model as BubbleTeaModel;
    ///
    /// struct App {
    ///     table: TableModel,
    /// }
    ///
    /// impl BubbleTeaModel for App {
    /// #   fn init() -> (Self, Option<bubbletea_rs::Cmd>) {
    /// #       (Self { table: TableModel::new(vec![]) }, None)
    /// #   }
    /// #   
    /// #   fn update(&mut self, _msg: bubbletea_rs::Msg) -> Option<bubbletea_rs::Cmd> {
    /// #       None
    /// #   }
    ///     
    ///     fn view(&self) -> String {
    ///         format!("My Application\n\n{}", self.table.view())
    ///     }
    /// }
    /// ```
    fn view(&self) -> String {
        self.viewport.view()
    }
}

/// Help system integration for displaying table navigation keys.
///
/// This implementation provides the help system with information about
/// the table's key bindings, enabling automatic generation of help text
/// that documents the available navigation commands.
impl help::KeyMap for Model {
    /// Returns the most commonly used key bindings for short help display.
    ///
    /// This method provides a concise list of the most essential navigation
    /// keys that users need to know for basic table operation. It's used
    /// when displaying compact help information.
    ///
    /// # Returns
    ///
    /// A vector of key binding references for row and page navigation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column};
    /// use bubbletea_widgets::help::KeyMap;
    ///
    /// let table = Model::new(vec![Column::new("Data", 20)]);
    /// let short_bindings = table.short_help();
    ///
    /// // Returns bindings for: up, down, page up, page down
    /// assert_eq!(short_bindings.len(), 4);
    /// ```
    ///
    /// # Help Content
    ///
    /// The short help includes:
    /// - **Row Up**: Move selection up one row
    /// - **Row Down**: Move selection down one row  
    /// - **Page Up**: Move up one page of rows
    /// - **Page Down**: Move down one page of rows
    fn short_help(&self) -> Vec<&key::Binding> {
        vec![
            &self.keymap.row_up,
            &self.keymap.row_down,
            &self.keymap.page_up,
            &self.keymap.page_down,
        ]
    }
    /// Returns all key bindings organized by category for full help display.
    ///
    /// This method provides a comprehensive list of all available navigation
    /// keys, organized into logical groups for clear presentation in detailed
    /// help displays. Each group contains related navigation commands.
    ///
    /// # Returns
    ///
    /// A vector of groups, where each group is a vector of related key bindings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::table::{Model, Column};
    /// use bubbletea_widgets::help::KeyMap;
    ///
    /// let table = Model::new(vec![Column::new("Data", 20)]);
    /// let full_bindings = table.full_help();
    ///
    /// // Returns 4 groups of key bindings
    /// assert_eq!(full_bindings.len(), 4);
    ///
    /// // First group: row navigation (up/down)
    /// assert_eq!(full_bindings[0].len(), 2);
    /// ```
    ///
    /// # Help Organization
    ///
    /// The full help is organized into these groups:
    /// 1. **Row Navigation**: Single row up/down movement
    /// 2. **Page Navigation**: Full page up/down scrolling
    /// 3. **Half Page Navigation**: Half page up/down movement
    /// 4. **Jump Navigation**: Go to start/end positions
    ///
    /// # Display Integration
    ///
    /// This grouped format allows help displays to show related commands
    /// together with appropriate spacing and categorization for better
    /// user comprehension.
    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![
            vec![&self.keymap.row_up, &self.keymap.row_down],
            vec![&self.keymap.page_up, &self.keymap.page_down],
            vec![&self.keymap.half_page_up, &self.keymap.half_page_down],
            vec![&self.keymap.go_to_start, &self.keymap.go_to_end],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cols() -> Vec<Column> {
        vec![
            Column::new("col1", 10),
            Column::new("col2", 10),
            Column::new("col3", 10),
        ]
    }

    #[test]
    fn test_new_defaults() {
        let m = Model::new(cols());
        assert_eq!(m.selected, 0);
        assert_eq!(m.height, 20);
    }

    #[test]
    fn test_with_rows() {
        let m = Model::new(cols()).with_rows(vec![Row::new(vec!["1".into(), "Foo".into()])]);
        assert_eq!(m.rows.len(), 1);
    }

    #[test]
    fn test_view_basic() {
        let mut m = Model::new(cols());
        m.set_height(5);
        m.rows = vec![Row::new(vec![
            "Foooooo".into(),
            "Baaaaar".into(),
            "Baaaaaz".into(),
        ])];
        let out = m.view();
        assert!(out.contains("Foooooo"));
    }
}
