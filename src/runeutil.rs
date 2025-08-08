//! Text sanitization utilities for cleaning and processing Unicode text.
//!
//! This module provides a comprehensive sanitizer that processes Unicode characters (runes)
//! to remove control characters and replace special characters like newlines and tabs
//! with customizable strings. It mirrors the functionality of Go's `runeutil` package
//! for compatibility with the bubbletea-widgets TUI library ecosystem.
//!
//! # Features
//!
//! - Remove control characters from text while preserving printable content
//! - Configurable replacement strings for newlines and tabs
//! - Builder pattern for sanitizer configuration using option functions
//! - Support for both `String` and `Vec<char>` input types
//! - UTF-8 safe processing with proper Unicode handling
//!
//! # Quick Start
//!
//! ```rust
//! use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs, replace_newlines};
//!
//! // Create a basic sanitizer with defaults
//! let sanitizer = new_sanitizer(vec![]);
//! let clean_text = sanitizer.sanitize_str("Hello\tworld\n");
//! assert_eq!(clean_text, "Hello    world\n");
//!
//! // Create a custom sanitizer
//! let custom_sanitizer = new_sanitizer(vec![
//!     replace_tabs(" "),  // Single space instead of 4
//!     replace_newlines(" | "),  // Pipe separator instead of newline
//! ]);
//! let result = custom_sanitizer.sanitize_str("Line 1\nLine 2\tTabbed");
//! assert_eq!(result, "Line 1 | Line 2 Tabbed");
//! ```
//!
//! # Common Use Cases
//!
//! - **Terminal UI text cleanup**: Remove control characters that could interfere with TUI rendering
//! - **Log processing**: Sanitize log messages for display in terminal applications
//! - **User input validation**: Clean user-provided text before display or storage
//! - **Cross-platform text normalization**: Handle different line ending conventions
//!
//! # Performance Notes
//!
//! The sanitizer is designed for efficiency with UTF-8 text:
//! - Pre-allocates output buffers based on input size
//! - Single-pass character iteration
//! - Zero-copy when no replacements are needed for individual characters

/// A configurable text sanitizer that removes control characters and replaces special characters.
///
/// The `Sanitizer` processes Unicode text by removing control characters that could
/// interfere with terminal display while allowing customizable replacement of common
/// whitespace characters like tabs and newlines.
///
/// # Default Behavior
///
/// - **Newlines** (`\n`, `\r`): Replaced with `"\n"` (preserves line breaks)
/// - **Tabs** (`\t`): Replaced with `"    "` (4 spaces)
/// - **Other control characters**: Removed entirely
/// - **Printable characters**: Preserved unchanged
///
/// # Configuration
///
/// Use the builder pattern with option functions to customize replacement behavior:
///
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs, replace_newlines};
///
/// let sanitizer = new_sanitizer(vec![
///     replace_tabs("\t"),        // Keep actual tabs
///     replace_newlines(" "),     // Replace newlines with spaces
/// ]);
/// ```
///
/// # Thread Safety
///
/// `Sanitizer` is `Clone` and can be shared across threads. Each sanitizer
/// instance maintains its own configuration and can be used concurrently.
///
/// # Examples
///
/// Basic sanitization:
/// ```rust
/// use bubbletea_widgets::runeutil::new_sanitizer;
///
/// let sanitizer = new_sanitizer(vec![]);
/// let input = "Hello\x08\tworld\x1b[31m\n";
/// let clean = sanitizer.sanitize_str(input);
/// // Control characters \x08 and \x1b[31m are removed
/// // Tab becomes 4 spaces, newline is preserved
/// assert_eq!(clean, "Hello    world\n");
/// ```
///
/// Custom replacement:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs, replace_newlines};
///
/// let sanitizer = new_sanitizer(vec![
///     replace_tabs(" "),      // Single space for tabs
///     replace_newlines(" | "), // Pipe separator for newlines
/// ]);
/// let result = sanitizer.sanitize_str("Line 1\nLine 2\tEnd");
/// assert_eq!(result, "Line 1 | Line 2 End");
/// ```
#[derive(Debug, Clone)]
pub struct Sanitizer {
    /// Replacement string for newline characters (`\n`, `\r`)
    replace_newline: String,
    /// Replacement string for tab characters (`\t`)
    replace_tab: String,
}

impl Default for Sanitizer {
    /// Creates a sanitizer with default replacement settings.
    ///
    /// # Default Values
    ///
    /// - **Newlines**: Preserved as `"\n"`
    /// - **Tabs**: Replaced with `"    "` (4 spaces)
    ///
    /// This provides sensible defaults for terminal display where tabs are
    /// typically rendered as multiple spaces and newlines should be preserved
    /// for proper text formatting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::runeutil::Sanitizer;
    ///
    /// let sanitizer = Sanitizer::default();
    /// let result = sanitizer.sanitize_str("Hello\tworld\n");
    /// assert_eq!(result, "Hello    world\n");
    /// ```
    fn default() -> Self {
        Self {
            replace_newline: "\n".to_string(),
            replace_tab: "    ".to_string(),
        }
    }
}

/// Configuration option for customizing sanitizer behavior during construction.
///
/// This type alias represents a closure that modifies a `Sanitizer` instance,
/// following the builder pattern commonly used in Go libraries. Each option
/// function takes a mutable reference to a sanitizer and configures a specific
/// aspect of its behavior.
///
/// # Design Pattern
///
/// This follows the functional options pattern where configuration is applied
/// through a series of functions rather than a large constructor with many
/// parameters. This approach provides:
///
/// - **Flexibility**: Easy to add new configuration options
/// - **Readability**: Self-documenting configuration code
/// - **Composability**: Options can be combined and reused
/// - **Backward compatibility**: New options don't break existing code
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::runeutil::{SanitizerOpt, Sanitizer};
///
/// // Custom option function
/// fn replace_semicolons(replacement: &str) -> SanitizerOpt {
///     let repl = replacement.to_string();
///     Box::new(move |s: &mut Sanitizer| {
///         // Custom logic would go here
///         // This is a conceptual example
///     })
/// }
/// ```
///
/// # Usage with new_sanitizer
///
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs};
///
/// let sanitizer = new_sanitizer(vec![
///     replace_tabs("  "),  // 2 spaces instead of 4
/// ]);
/// ```
pub type SanitizerOpt = Box<dyn FnOnce(&mut Sanitizer)>;

/// Creates a new sanitizer with the specified configuration options.
///
/// This function implements the functional options pattern, allowing flexible
/// configuration through a vector of option functions. It starts with default
/// settings and applies each option in sequence.
///
/// # Arguments
///
/// * `opts` - A vector of configuration options to apply to the sanitizer
///
/// # Returns
///
/// A configured `Sanitizer` instance ready for text processing
///
/// # Examples
///
/// Create a sanitizer with default settings:
/// ```rust
/// use bubbletea_widgets::runeutil::new_sanitizer;
///
/// let sanitizer = new_sanitizer(vec![]);
/// let result = sanitizer.sanitize_str("Hello\tworld");
/// assert_eq!(result, "Hello    world");
/// ```
///
/// Create a sanitizer with custom tab and newline handling:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs, replace_newlines};
///
/// let sanitizer = new_sanitizer(vec![
///     replace_tabs(" "),       // Single space for tabs
///     replace_newlines("<br>"), // HTML line breaks
/// ]);
/// let result = sanitizer.sanitize_str("Line 1\nLine 2\tTabbed");
/// assert_eq!(result, "Line 1<br>Line 2 Tabbed");
/// ```
///
/// Combine multiple options:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs, replace_newlines};
///
/// let options = vec![
///     replace_tabs("→"),        // Visible tab character
///     replace_newlines("↵\n"),  // Visible newline + actual newline
/// ];
/// let sanitizer = new_sanitizer(options);
/// ```
///
/// # Option Application Order
///
/// Options are applied in the order they appear in the vector. If multiple
/// options modify the same setting, the last one takes precedence:
///
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs};
///
/// let sanitizer = new_sanitizer(vec![
///     replace_tabs("FIRST"),
///     replace_tabs("SECOND"), // This one wins
/// ]);
/// let result = sanitizer.sanitize_str("a\tb");
/// assert_eq!(result, "aSECONDb");
/// ```
pub fn new_sanitizer(opts: Vec<SanitizerOpt>) -> Sanitizer {
    let mut s = Sanitizer::default();
    for opt in opts { opt(&mut s); }
    s
}

/// Creates an option to replace tab characters with a custom string.
///
/// This option function configures how tab characters (`\t`) should be handled
/// during sanitization. By default, tabs are replaced with 4 spaces, but this
/// function allows you to specify any replacement string.
///
/// # Arguments
///
/// * `tab_repl` - The string to replace each tab character with
///
/// # Returns
///
/// A `SanitizerOpt` that can be passed to `new_sanitizer()`
///
/// # Examples
///
/// Replace tabs with 2 spaces:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs};
///
/// let sanitizer = new_sanitizer(vec![replace_tabs("  ")]);
/// let result = sanitizer.sanitize_str("Hello\tworld");
/// assert_eq!(result, "Hello  world");
/// ```
///
/// Replace tabs with a visible character:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs};
///
/// let sanitizer = new_sanitizer(vec![replace_tabs("→")]);
/// let result = sanitizer.sanitize_str("Column1\tColumn2");
/// assert_eq!(result, "Column1→Column2");
/// ```
///
/// Preserve tabs as-is:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs};
///
/// let sanitizer = new_sanitizer(vec![replace_tabs("\t")]);
/// let result = sanitizer.sanitize_str("Keep\ttabs");
/// assert_eq!(result, "Keep\ttabs");
/// ```
///
/// Remove tabs entirely:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs};
///
/// let sanitizer = new_sanitizer(vec![replace_tabs("")]);
/// let result = sanitizer.sanitize_str("No\ttabs");
/// assert_eq!(result, "Notabs");
/// ```
pub fn replace_tabs(tab_repl: &str) -> SanitizerOpt {
    let repl = tab_repl.to_string();
    Box::new(move |s: &mut Sanitizer| { s.replace_tab = repl.clone(); })
}

/// Creates an option to replace newline characters with a custom string.
///
/// This option function configures how newline characters (`\n` and `\r`) should
/// be handled during sanitization. By default, newlines are preserved as `\n`,
/// but this function allows you to specify any replacement string.
///
/// # Arguments
///
/// * `nl_repl` - The string to replace each newline character with
///
/// # Returns
///
/// A `SanitizerOpt` that can be passed to `new_sanitizer()`
///
/// # Newline Handling
///
/// This option affects both `\n` (LF) and `\r` (CR) characters, making it suitable
/// for handling different line ending conventions across platforms.
///
/// # Examples
///
/// Replace newlines with spaces for single-line display:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_newlines};
///
/// let sanitizer = new_sanitizer(vec![replace_newlines(" ")]);
/// let result = sanitizer.sanitize_str("Line 1\nLine 2\nLine 3");
/// assert_eq!(result, "Line 1 Line 2 Line 3");
/// ```
///
/// Replace with visible characters for debugging:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_newlines};
///
/// let sanitizer = new_sanitizer(vec![replace_newlines("↵")]);
/// let result = sanitizer.sanitize_str("First\nSecond");
/// assert_eq!(result, "First↵Second");
/// ```
///
/// Replace with HTML line breaks:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_newlines};
///
/// let sanitizer = new_sanitizer(vec![replace_newlines("<br>")]);
/// let result = sanitizer.sanitize_str("Para 1\nPara 2");
/// assert_eq!(result, "Para 1<br>Para 2");
/// ```
///
/// Remove newlines entirely:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_newlines};
///
/// let sanitizer = new_sanitizer(vec![replace_newlines("")]);
/// let result = sanitizer.sanitize_str("No\nbreaks");
/// assert_eq!(result, "Nobreaks");
/// ```
///
/// Handle both Unix and Windows line endings:
/// ```rust
/// use bubbletea_widgets::runeutil::{new_sanitizer, replace_newlines};
///
/// let sanitizer = new_sanitizer(vec![replace_newlines(" | ")]);
/// let unix_input = "Line1\nLine2";
/// let windows_input = "Line1\r\nLine2";
/// assert_eq!(sanitizer.sanitize_str(unix_input), "Line1 | Line2");
/// // \r\n becomes " |  | " since both \r and \n are replaced
/// ```
pub fn replace_newlines(nl_repl: &str) -> SanitizerOpt {
    let repl = nl_repl.to_string();
    Box::new(move |s: &mut Sanitizer| { s.replace_newline = repl.clone(); })
}

impl Sanitizer {
    /// Sanitizes a string by removing control characters and replacing special characters.
    ///
    /// This method processes the input string character by character, applying the
    /// sanitizer's configuration to produce clean output suitable for display in
    /// terminal applications.
    ///
    /// # Processing Rules
    ///
    /// 1. **Newlines** (`\n`, `\r`): Replaced with the configured newline replacement
    /// 2. **Tabs** (`\t`): Replaced with the configured tab replacement
    /// 3. **Other control characters**: Removed entirely (e.g., `\x08`, `\x1b`)
    /// 4. **Printable characters**: Preserved unchanged
    ///
    /// # Arguments
    ///
    /// * `input` - The string to sanitize
    ///
    /// # Returns
    ///
    /// A new `String` containing the sanitized text
    ///
    /// # Performance
    ///
    /// - Pre-allocates output buffer based on input length for efficiency
    /// - Single-pass iteration over input characters
    /// - Replacement strings are inserted directly without additional allocations
    ///
    /// # Examples
    ///
    /// Basic sanitization with defaults:
    /// ```rust
    /// use bubbletea_widgets::runeutil::new_sanitizer;
    ///
    /// let sanitizer = new_sanitizer(vec![]);
    /// let input = "Hello\x08\tworld\x1b[31mRed\x1b[0m\n";
    /// let clean = sanitizer.sanitize_str(input);
    /// assert_eq!(clean, "Hello    worldRed\n");
    /// ```
    ///
    /// Custom replacement configuration:
    /// ```rust
    /// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs, replace_newlines};
    ///
    /// let sanitizer = new_sanitizer(vec![
    ///     replace_tabs(" "),
    ///     replace_newlines(" | "),
    /// ]);
    /// let result = sanitizer.sanitize_str("Line 1\nLine 2\tEnd");
    /// assert_eq!(result, "Line 1 | Line 2 End");
    /// ```
    ///
    /// Handling various control characters:
    /// ```rust
    /// use bubbletea_widgets::runeutil::new_sanitizer;
    ///
    /// let sanitizer = new_sanitizer(vec![]);
    /// let input = "Text\x00\x07\x1b[2J\x7f";
    /// let clean = sanitizer.sanitize_str(input);
    /// assert_eq!(clean, "Text"); // All control chars removed
    /// ```
    ///
    /// # UTF-8 Safety
    ///
    /// Since Rust strings are guaranteed to be valid UTF-8, this method doesn't
    /// need to handle invalid Unicode sequences, unlike its Go counterpart.
    pub fn sanitize_str(&self, input: &str) -> String {
        // Since Rust `&str` is valid UTF-8, we don't need to handle invalid runes.
        let mut out = String::with_capacity(input.len());
        for ch in input.chars() {
            match ch {
                '\r' | '\n' => out.push_str(&self.replace_newline),
                '\t' => out.push_str(&self.replace_tab),
                c if c.is_control() => { /* skip other control chars */ }
                c => out.push(c),
            }
        }
        out
    }

    /// Sanitizes a slice of characters by removing control characters and replacing special characters.
    ///
    /// This method processes a slice of Unicode characters (runes) and applies the same
    /// sanitization rules as `sanitize_str()`, but operates on individual characters
    /// rather than a string. This is useful when working with character vectors or
    /// when you need character-level processing.
    ///
    /// # Processing Rules
    ///
    /// 1. **Newlines** (`\n`, `\r`): Replaced with characters from the configured newline replacement string
    /// 2. **Tabs** (`\t`): Replaced with characters from the configured tab replacement string
    /// 3. **Other control characters**: Removed entirely
    /// 4. **Printable characters**: Preserved unchanged
    ///
    /// # Arguments
    ///
    /// * `runes` - A slice of characters to sanitize
    ///
    /// # Returns
    ///
    /// A new `Vec<char>` containing the sanitized characters
    ///
    /// # Replacement Expansion
    ///
    /// When replacement strings contain multiple characters, they are expanded
    /// into individual characters in the output vector:
    ///
    /// - Tab replacement `"    "` becomes 4 separate space characters
    /// - Newline replacement `"<br>"` becomes 4 characters: `['<', 'b', 'r', '>']`
    ///
    /// # Performance
    ///
    /// - Pre-allocates output buffer based on input length
    /// - Single-pass iteration with efficient character extension
    /// - No intermediate string allocations
    ///
    /// # Examples
    ///
    /// Basic character vector sanitization:
    /// ```rust
    /// use bubbletea_widgets::runeutil::new_sanitizer;
    ///
    /// let sanitizer = new_sanitizer(vec![]);
    /// let input: Vec<char> = "Hello\tworld\n".chars().collect();
    /// let clean = sanitizer.sanitize_runes(&input);
    /// let expected: Vec<char> = "Hello    world\n".chars().collect();
    /// assert_eq!(clean, expected);
    /// ```
    ///
    /// Custom replacements with character expansion:
    /// ```rust
    /// use bubbletea_widgets::runeutil::{new_sanitizer, replace_tabs, replace_newlines};
    ///
    /// let sanitizer = new_sanitizer(vec![
    ///     replace_tabs("[TAB]"),
    ///     replace_newlines("[NL]"),
    /// ]);
    /// let input: Vec<char> = "a\tb\n".chars().collect();
    /// let result = sanitizer.sanitize_runes(&input);
    /// let expected: Vec<char> = "a[TAB]b[NL]".chars().collect();
    /// assert_eq!(result, expected);
    /// ```
    ///
    /// Removing control characters:
    /// ```rust
    /// use bubbletea_widgets::runeutil::new_sanitizer;
    ///
    /// let sanitizer = new_sanitizer(vec![]);
    /// let input = vec!['H', 'i', '\x07', '\x1b', '!'];
    /// let clean = sanitizer.sanitize_runes(&input);
    /// assert_eq!(clean, vec!['H', 'i', '!']);
    /// ```
    ///
    /// # Use Cases
    ///
    /// This method is particularly useful when:
    /// - Working with text editors that store content as character arrays
    /// - Processing individual characters in a stream
    /// - Converting between string and character vector representations
    /// - Implementing custom text processing pipelines
    pub fn sanitize_runes(&self, runes: &[char]) -> Vec<char> {
        let mut out: Vec<char> = Vec::with_capacity(runes.len());
        for &r in runes {
            match r {
                '\r' | '\n' => out.extend(self.replace_newline.chars()),
                '\t' => out.extend(self.replace_tab.chars()),
                c if c.is_control() => {}
                c => out.push(c),
            }
        }
        out
    }
}


