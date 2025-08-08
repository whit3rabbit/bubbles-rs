//! Progress component for Bubble Tea applications.
//!
//! Package progress provides a simple progress bar for Bubble Tea applications.
//! It closely matches the Go bubbles progress component API for 1-1 compatibility.
//!
//! # Basic Usage
//!
//! ```rust
//! use bubbles_rs::progress::{new, with_width, with_solid_fill};
//!
//! // Create a progress bar with default settings
//! let progress = new(&[]);
//!
//! // Create a progress bar with custom settings using the option pattern
//! let progress = new(&[
//!     with_width(50),
//!     with_solid_fill("#ff0000".to_string()),
//! ]);
//! ```
//!
//! # Animation and Control
//!
//! ```rust
//! use bubbles_rs::progress::new;
//!
//! let mut progress = new(&[]);
//!
//! // Set progress (returns command for animation)
//! let cmd = progress.set_percent(0.75); // 75%
//! let cmd = progress.incr_percent(0.1); // Add 10%
//! let cmd = progress.decr_percent(0.05); // Subtract 5%
//! ```

use bubbletea_rs::{tick as bubbletea_tick, Cmd, Model as BubbleTeaModel, Msg};
use lipgloss::blending::blend_1d;
use lipgloss::Color as LGColor;
use lipgloss::Style;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

// Internal ID management for progress instances
static LAST_ID: AtomicI64 = AtomicI64::new(0);

fn next_id() -> i64 {
    LAST_ID.fetch_add(1, Ordering::SeqCst) + 1
}

// Constants matching Go implementation
const FPS: u32 = 60;
const DEFAULT_WIDTH: i32 = 40;
const DEFAULT_FREQUENCY: f64 = 18.0;
const DEFAULT_DAMPING: f64 = 1.0;

/// Configuration options for customizing progress bar behavior and appearance.
///
/// This enum provides a builder pattern for configuring progress bars with various
/// visual and behavioral options. Options can be combined to create highly customized
/// progress bars that match your application's design and functionality needs.
///
/// # Examples
///
/// ## Basic Customization
/// ```rust
/// use bubbles_rs::progress::{new, with_width, with_solid_fill};
///
/// let progress = new(&[
///     with_width(60),
///     with_solid_fill("#00ff00".to_string()),
/// ]);
/// ```
///
/// ## Advanced Gradient Configuration
/// ```rust
/// use bubbles_rs::progress::{new, with_gradient, with_width, without_percentage};
///
/// let gradient_progress = new(&[
///     with_width(80),
///     with_gradient("#ff4757".to_string(), "#5352ed".to_string()),
///     without_percentage(),
/// ]);
/// ```
pub enum ProgressOption {
    /// Uses the default gradient colors (purple to pink).
    /// Creates a smooth color transition from #5A56E0 to #EE6FF8.
    WithDefaultGradient,
    /// Creates a custom gradient between two specified colors.
    /// The first string is the start color, the second is the end color.
    /// Colors can be hex codes ("#ff0000") or named colors.
    WithGradient(String, String),
    /// Uses the default gradient colors but scales the gradient to fit only the filled portion.
    /// This creates a more dynamic visual effect where the gradient adjusts based on progress.
    WithDefaultScaledGradient,
    /// Creates a custom scaled gradient that fits only the filled portion of the bar.
    /// Combines custom colors with dynamic gradient scaling for maximum visual impact.
    WithScaledGradient(String, String),
    /// Sets a solid color fill instead of a gradient.
    /// Provides consistent coloring across the entire filled portion.
    WithSolidFill(String),
    /// Customizes the characters used for filled and empty portions of the progress bar.
    /// First character is for filled sections, second for empty sections.
    WithFillCharacters(char, char),
    /// Hides the percentage text display.
    /// Useful when you want a cleaner look or when space is limited.
    WithoutPercentage,
    /// Sets the total width of the progress bar in characters.
    /// This includes both the bar and percentage text if shown.
    WithWidth(i32),
    /// Configures the spring animation parameters for smooth transitions.
    /// First value is frequency (speed), second is damping (bounciness).
    WithSpringOptions(f64, f64),
}

impl ProgressOption {
    fn apply(&self, m: &mut Model) {
        match self {
            ProgressOption::WithDefaultGradient => {
                m.set_ramp("#5A56E0".to_string(), "#EE6FF8".to_string(), false);
            }
            ProgressOption::WithGradient(color_a, color_b) => {
                m.set_ramp(color_a.clone(), color_b.clone(), false);
            }
            ProgressOption::WithDefaultScaledGradient => {
                m.set_ramp("#5A56E0".to_string(), "#EE6FF8".to_string(), true);
            }
            ProgressOption::WithScaledGradient(color_a, color_b) => {
                m.set_ramp(color_a.clone(), color_b.clone(), true);
            }
            ProgressOption::WithSolidFill(color) => {
                m.full_color = color.clone();
                m.use_ramp = false;
            }
            ProgressOption::WithFillCharacters(full, empty) => {
                m.full = *full;
                m.empty = *empty;
            }
            ProgressOption::WithoutPercentage => {
                m.show_percentage = false;
            }
            ProgressOption::WithWidth(width) => {
                m.width = *width;
            }
            ProgressOption::WithSpringOptions(frequency, damping) => {
                m.set_spring_options(*frequency, *damping);
                m.spring_customized = true;
            }
        }
    }
}

/// Creates a gradient fill with default colors.
///
/// Uses the predefined gradient colors (#5A56E0 to #EE6FF8) that provide
/// an attractive purple-to-pink transition. This is a convenient option
/// for getting a professional-looking gradient without specifying colors.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::{new, with_default_gradient, with_width};
///
/// let progress = new(&[
///     with_default_gradient(),
///     with_width(50),
/// ]);
/// ```
pub fn with_default_gradient() -> ProgressOption {
    ProgressOption::WithDefaultGradient
}

/// Creates a custom gradient fill blending between two specified colors.
///
/// The gradient transitions smoothly from the first color to the second color
/// across the width of the progress bar. Colors can be specified as hex codes
/// (e.g., "#ff0000") or named colors supported by your terminal.
///
/// # Arguments
///
/// * `color_a` - The starting color of the gradient
/// * `color_b` - The ending color of the gradient
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::{new, with_gradient};
///
/// // Red to blue gradient
/// let progress = new(&[
///     with_gradient("#ff0000".to_string(), "#0000ff".to_string()),
/// ]);
///
/// // Green to yellow gradient
/// let warm_progress = new(&[
///     with_gradient("#10ac84".to_string(), "#f9ca24".to_string()),
/// ]);
/// ```
pub fn with_gradient(color_a: String, color_b: String) -> ProgressOption {
    ProgressOption::WithGradient(color_a, color_b)
}

/// Creates a scaled gradient with default colors.
///
/// Similar to `with_default_gradient()` but scales the gradient to fit only
/// the filled portion of the progress bar. This creates a more dynamic effect
/// where the gradient adjusts its range based on the current progress level.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::{new, with_default_scaled_gradient};
///
/// let progress = new(&[
///     with_default_scaled_gradient(),
/// ]);
///
/// // At 50% progress, the gradient spans only the filled half
/// // At 100% progress, the gradient spans the entire bar
/// ```
pub fn with_default_scaled_gradient() -> ProgressOption {
    ProgressOption::WithDefaultScaledGradient
}

/// Creates a custom scaled gradient that fits the filled portion width.
///
/// Combines custom color selection with dynamic gradient scaling. The gradient
/// transitions from the first color to the second color across only the filled
/// portion of the progress bar, creating an adaptive visual effect.
///
/// # Arguments
///
/// * `color_a` - The starting color of the scaled gradient
/// * `color_b` - The ending color of the scaled gradient
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::{new, with_scaled_gradient};
///
/// let progress = new(&[
///     with_scaled_gradient("#ee5a24".to_string(), "#feca57".to_string()),
/// ]);
///
/// // The orange-to-yellow gradient will always span the filled portion,
/// // regardless of progress percentage
/// ```
pub fn with_scaled_gradient(color_a: String, color_b: String) -> ProgressOption {
    ProgressOption::WithScaledGradient(color_a, color_b)
}

/// Sets the progress bar to use a solid color fill.
///
/// Instead of a gradient, this option fills the progress bar with a single,
/// consistent color. This provides a clean, minimalist appearance and can
/// be useful for maintaining consistency with your application's color scheme.
///
/// # Arguments
///
/// * `color` - The color to use for the filled portion (hex code or named color)
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::{new, with_solid_fill, with_width};
///
/// // Solid green progress bar
/// let success_progress = new(&[
///     with_solid_fill("#2ed573".to_string()),
///     with_width(40),
/// ]);
///
/// // Solid red for error states
/// let error_progress = new(&[
///     with_solid_fill("#ff3838".to_string()),
/// ]);
/// ```
pub fn with_solid_fill(color: String) -> ProgressOption {
    ProgressOption::WithSolidFill(color)
}

/// Customizes the characters used for filled and empty sections.
///
/// Allows you to change the visual representation of the progress bar by
/// specifying different characters for the filled and empty portions. This
/// can be used to create different visual styles or to match specific design requirements.
///
/// # Arguments
///
/// * `full` - Character to use for filled sections (default: '█')
/// * `empty` - Character to use for empty sections (default: '░')
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::{new, with_fill_characters};
///
/// // Classic ASCII style
/// let ascii_progress = new(&[
///     with_fill_characters('=', '-'),
/// ]);
///
/// // Block style with different densities
/// let block_progress = new(&[
///     with_fill_characters('▓', '▒'),
/// ]);
///
/// // Dot style
/// let dot_progress = new(&[
///     with_fill_characters('●', '○'),
/// ]);
/// ```
pub fn with_fill_characters(full: char, empty: char) -> ProgressOption {
    ProgressOption::WithFillCharacters(full, empty)
}

/// Hides the numeric percentage display.
///
/// By default, progress bars show a percentage (e.g., " 75%") alongside
/// the visual bar. This option removes the percentage text, creating a
/// cleaner appearance and saving horizontal space.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::{new, without_percentage, with_width, with_solid_fill};
///
/// // Clean progress bar without percentage text
/// let minimal_progress = new(&[
///     without_percentage(),
///     with_width(30),
/// ]);
///
/// // Useful for compact layouts
/// let compact_progress = new(&[
///     without_percentage(),
///     with_solid_fill("#3742fa".to_string()),
/// ]);
/// ```
pub fn without_percentage() -> ProgressOption {
    ProgressOption::WithoutPercentage
}

/// Sets the total width of the progress bar in characters.
///
/// This width includes both the visual bar and the percentage text (if shown).
/// You can also modify the width later using the `width` field, which is useful
/// for responsive layouts that need to adjust to terminal size changes.
///
/// # Arguments
///
/// * `w` - Width in characters (must be positive)
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::{new, with_width};
///
/// // Narrow progress bar for compact spaces
/// let narrow = new(&[with_width(20)]);
///
/// // Wide progress bar for detailed view
/// let wide = new(&[with_width(80)]);
///
/// // Responsive width (can be changed later)
/// let mut responsive = new(&[with_width(40)]);
/// responsive.width = 60; // Adjust based on terminal size
/// ```
pub fn with_width(w: i32) -> ProgressOption {
    ProgressOption::WithWidth(w)
}

/// Configures the spring animation parameters for smooth progress transitions.
///
/// The progress bar uses a spring-based physics system to animate between
/// different progress values. This creates natural-looking transitions that
/// feel responsive and smooth.
///
/// # Arguments
///
/// * `frequency` - Animation speed (higher = faster, typical range: 10-30)
/// * `damping` - Bounciness control (higher = less bouncy, typical range: 0.5-2.0)
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::{new, with_spring_options};
///
/// // Fast, snappy animation
/// let snappy = new(&[
///     with_spring_options(25.0, 1.5),
/// ]);
///
/// // Slow, bouncy animation
/// let bouncy = new(&[
///     with_spring_options(12.0, 0.7),
/// ]);
///
/// // Smooth, professional animation
/// let smooth = new(&[
///     with_spring_options(18.0, 1.2),
/// ]);
/// ```
pub fn with_spring_options(frequency: f64, damping: f64) -> ProgressOption {
    ProgressOption::WithSpringOptions(frequency, damping)
}

/// Message indicating that an animation frame should be processed.
///
/// This message is used internally by the progress bar's animation system to
/// trigger smooth transitions between progress values. Each `FrameMsg` is
/// associated with a specific progress bar instance and animation sequence
/// to ensure proper message routing and prevent timing conflicts.
///
/// The message contains identity information that allows the progress bar
/// to validate that it should process the frame update, preventing issues
/// with multiple progress bars or rapid state changes.
///
/// # Internal Usage
///
/// You typically won't create `FrameMsg` instances directly. They are
/// generated automatically when you call methods like `set_percent()`,
/// `incr_percent()`, or `decr_percent()` on a progress bar.
///
/// # Examples
///
/// ```rust
/// use bubbles_rs::progress::new;
///
/// let mut progress = new(&[]);
///
/// // This automatically creates and schedules FrameMsg instances
/// let cmd = progress.set_percent(0.75);
///
/// // The progress bar's update() method will handle the FrameMsg
/// // to animate smoothly to 75%
/// ```
#[derive(Debug, Clone)]
pub struct FrameMsg {
    /// Unique identifier of the progress bar instance.
    ///
    /// This ensures that frame messages are only processed by the
    /// correct progress bar when multiple bars exist in an application.
    id: i64,
    /// Animation sequence tag to prevent stale frame messages.
    ///
    /// The tag is incremented each time a new animation starts,
    /// allowing the progress bar to ignore outdated frame messages
    /// from previous animation sequences.
    tag: i64,
}

/// Simple spring animation system (simplified version of harmonica)
#[derive(Debug, Clone)]
struct Spring {
    frequency: f64,
    damping: f64,
    fps: f64,
}

impl Spring {
    fn new(fps: f64, frequency: f64, damping: f64) -> Self {
        Self {
            frequency,
            damping,
            fps,
        }
    }

    fn update(&self, position: f64, velocity: f64, target: f64) -> (f64, f64) {
        let dt = 1.0 / self.fps;
        let spring_force = -self.frequency * (position - target);
        let damping_force = -self.damping * velocity;
        let acceleration = spring_force + damping_force;

        let new_velocity = velocity + acceleration * dt;
        let new_position = position + new_velocity * dt;

        (new_position, new_velocity)
    }
}

/// The main progress bar model containing all state and configuration.
///
/// This structure holds all the data needed to render and animate a progress bar,
/// including visual styling, animation state, and behavioral configuration. The model
/// follows the Elm Architecture pattern used by bubbletea-rs, with separate methods
/// for initialization, updates, and rendering.
///
/// # Key Features
///
/// - **Smooth animation** using spring-based physics for natural transitions
/// - **Flexible styling** with support for gradients, solid colors, and custom characters
/// - **Responsive design** with configurable width and percentage display options
/// - **Thread safety** with unique identifiers for multi-instance usage
///
/// # Animation System
///
/// The progress bar uses a spring physics model to create smooth, natural-looking
/// transitions between progress values. This provides better visual feedback than
/// instant jumps and makes progress changes feel more responsive and polished.
///
/// # Examples
///
/// ## Basic Usage
/// ```rust
/// use bubbles_rs::progress::{new, with_width};
///
/// let mut progress = new(&[with_width(50)]);
///
/// // Set progress and get animation command
/// let cmd = progress.set_percent(0.6);
///
/// // Render the current state
/// let view = progress.view();
/// println!("{}", view); // Shows animated progress bar
/// ```
///
/// ## Advanced Configuration
/// ```rust
/// use bubbles_rs::progress::*;
///
/// let mut progress = new(&[
///     with_width(60),
///     with_gradient("#ff6b6b".to_string(), "#4ecdc4".to_string()),
///     with_spring_options(20.0, 1.0),
/// ]);
///
/// // Smooth animated increment
/// let cmd = progress.incr_percent(0.1);
/// ```
///
/// ## Integration with bubbletea-rs
/// ```rust
/// use bubbles_rs::progress;
/// use bubbletea_rs::{Model as TeaModel, Cmd, Msg};
///
/// struct App {
///     progress: progress::Model,
/// }
///
/// impl TeaModel for App {
///     fn init() -> (Self, Option<Cmd>) {
///         let progress = progress::new(&[
///             progress::with_width(40),
///             progress::with_solid_fill("#2ecc71".to_string()),
///         ]);
///         (Self { progress }, None)
///     }
///
///     fn update(&mut self, msg: Msg) -> Option<Cmd> {
///         // Forward animation messages to progress bar
///         self.progress.update(msg)
///     }
///
///     fn view(&self) -> String {
///         format!("Loading: {}\n", self.progress.view())
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Model {
    /// An identifier to keep us from receiving messages intended for other
    /// progress bars.
    id: i64,

    /// An identifier to keep us from receiving frame messages too quickly.
    tag: i64,

    /// Total width of the progress bar, including percentage, if set.
    pub width: i32,

    /// "Filled" sections of the progress bar.
    pub full: char,
    /// Color used for the filled portion (hex or named color string).
    pub full_color: String,

    /// "Empty" sections of the progress bar.
    pub empty: char,
    /// Color used for the empty portion (hex or named color string).
    pub empty_color: String,

    /// Settings for rendering the numeric percentage.
    pub show_percentage: bool,
    /// Format string for the percentage (e.g., " %3.0f%%").
    pub percent_format: String,
    /// Lipgloss style applied to the percentage text.
    pub percentage_style: Style,

    /// Members for animated transitions.
    spring: Spring,
    spring_customized: bool,
    percent_shown: f64,  // percent currently displaying
    target_percent: f64, // percent to which we're animating
    velocity: f64,

    /// Gradient settings
    use_ramp: bool,
    ramp_color_a: String, // simplified color handling compared to Go's colorful
    ramp_color_b: String,

    /// When true, we scale the gradient to fit the width of the filled section
    /// of the progress bar. When false, the width of the gradient will be set
    /// to the full width of the progress bar.
    scale_ramp: bool,
}

/// Creates a new progress bar with the specified configuration options.
///
/// This function initializes a progress bar with sensible defaults and applies
/// any provided options to customize its appearance and behavior. The progress bar
/// starts at 0% and is ready for animation and display.
///
/// # Arguments
///
/// * `opts` - A slice of `ProgressOption` values to configure the progress bar
///
/// # Default Configuration
///
/// - **Width**: 40 characters
/// - **Fill character**: '█' (full block)
/// - **Empty character**: '░' (light shade)
/// - **Fill color**: "#7571F9" (purple)
/// - **Empty color**: "#606060" (gray)
/// - **Percentage**: Shown by default
/// - **Animation**: Spring physics with frequency=18.0, damping=1.0
///
/// # Examples
///
/// ## Basic Progress Bar
/// ```rust
/// use bubbles_rs::progress::new;
///
/// // Create with all defaults
/// let progress = new(&[]);
/// assert_eq!(progress.width, 40);
/// ```
///
/// ## Customized Progress Bar
/// ```rust
/// use bubbles_rs::progress::*;
///
/// let progress = new(&[
///     with_width(60),
///     with_solid_fill("#e74c3c".to_string()),
///     without_percentage(),
///     with_spring_options(25.0, 0.8),
/// ]);
/// ```
///
/// ## Gradient Progress Bar
/// ```rust
/// use bubbles_rs::progress::*;
///
/// let gradient_progress = new(&[
///     with_gradient("#667eea".to_string(), "#764ba2".to_string()),
///     with_width(50),
/// ]);
/// ```
pub fn new(opts: &[ProgressOption]) -> Model {
    let mut m = Model {
        id: next_id(),
        tag: 0,
        width: DEFAULT_WIDTH,
        full: '█',
        full_color: "#7571F9".to_string(),
        empty: '░',
        empty_color: "#606060".to_string(),
        show_percentage: true,
        percent_format: " %3.0f%%".to_string(),
        percentage_style: Style::new(),
        spring: Spring::new(FPS as f64, DEFAULT_FREQUENCY, DEFAULT_DAMPING),
        spring_customized: false,
        percent_shown: 0.0,
        target_percent: 0.0,
        velocity: 0.0,
        use_ramp: false,
        ramp_color_a: String::new(),
        ramp_color_b: String::new(),
        scale_ramp: false,
    };

    for opt in opts {
        opt.apply(&mut m);
    }

    if !m.spring_customized {
        m.set_spring_options(DEFAULT_FREQUENCY, DEFAULT_DAMPING);
    }

    m
}

/// NewModel returns a model with default values.
/// Deprecated: use [new] instead.
#[deprecated(since = "0.0.7", note = "use new instead")]
pub fn new_model(opts: &[ProgressOption]) -> Model {
    new(opts)
}

impl Model {
    /// Configures the spring animation parameters for smooth transitions.
    ///
    /// Updates the internal spring physics engine that controls how the progress bar
    /// animates between different progress values. Higher frequency values make
    /// animations faster, while higher damping values reduce bounciness.
    ///
    /// # Arguments
    ///
    /// * `frequency` - Animation speed (typical range: 10.0-30.0)
    /// * `damping` - Bounciness control (typical range: 0.5-2.0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let mut progress = new(&[]);
    ///
    /// // Fast, snappy animation
    /// progress.set_spring_options(25.0, 1.5);
    ///
    /// // Slow, bouncy animation  
    /// progress.set_spring_options(12.0, 0.7);
    ///
    /// // Now progress changes will use the new animation style
    /// let cmd = progress.set_percent(0.8);
    /// ```
    pub fn set_spring_options(&mut self, frequency: f64, damping: f64) {
        self.spring = Spring::new(FPS as f64, frequency, damping);
    }

    /// Returns the target percentage that the progress bar is animating towards.
    ///
    /// This represents the logical progress value, not necessarily what's currently
    /// being displayed. During animation, the visual progress gradually moves from
    /// its current position toward this target value using spring physics.
    ///
    /// # Returns
    ///
    /// The target progress as a float between 0.0 and 1.0 (0% to 100%).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let mut progress = new(&[]);
    /// assert_eq!(progress.percent(), 0.0);
    ///
    /// // Set target to 75%
    /// progress.set_percent(0.75);
    /// assert_eq!(progress.percent(), 0.75);
    ///
    /// // The visual bar will animate smoothly to reach this target
    /// ```
    pub fn percent(&self) -> f64 {
        self.target_percent
    }

    /// Sets the progress to a specific percentage and returns an animation command.
    ///
    /// This method updates the target percentage and initiates a smooth animation
    /// to the new value using spring physics. The returned command should be
    /// handled by your bubbletea-rs application to drive the animation.
    ///
    /// # Arguments
    ///
    /// * `p` - Progress percentage as a float (will be clamped to 0.0-1.0 range)
    ///
    /// # Returns
    ///
    /// A `Cmd` that drives the animation. This must be returned from your
    /// application's `update()` method to enable smooth progress transitions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::progress::new;
    /// use bubbletea_rs::{Model, Msg, Cmd};
    ///
    /// struct App {
    ///     progress: bubbles_rs::progress::Model,
    /// }
    ///
    /// impl Model for App {
    ///     fn update(&mut self, msg: Msg) -> Option<Cmd> {
    ///         // Handle progress animation
    ///         if let Some(cmd) = self.progress.update(msg) {
    ///             return Some(cmd);
    ///         }
    ///
    ///         // Set progress and return animation command
    ///         Some(self.progress.set_percent(0.6))
    ///     }
    /// #   fn init() -> (Self, Option<Cmd>) { (Self { progress: new(&[]) }, None) }
    /// #   fn view(&self) -> String { String::new() }
    /// }
    /// ```
    ///
    /// ## Direct Usage
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let mut progress = new(&[]);
    ///
    /// // Values are automatically clamped
    /// let cmd1 = progress.set_percent(0.5);   // 50%
    /// let cmd2 = progress.set_percent(1.5);   // Clamped to 100%
    /// let cmd3 = progress.set_percent(-0.2);  // Clamped to 0%
    /// ```
    pub fn set_percent(&mut self, p: f64) -> Cmd {
        self.target_percent = p.clamp(0.0, 1.0);
        self.tag += 1;
        self.next_frame()
    }

    /// Increases the progress by a specified amount and returns an animation command.
    ///
    /// This is a convenience method that adds the given value to the current
    /// progress percentage. The result is automatically clamped to the valid
    /// range (0.0-1.0) and animated smoothly to the new position.
    ///
    /// # Arguments
    ///
    /// * `v` - Amount to add to current progress (can be negative for decrease)
    ///
    /// # Returns
    ///
    /// A `Cmd` that drives the animation to the new progress value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let mut progress = new(&[]);
    ///
    /// // Start at 0%, increment by 25%
    /// let cmd1 = progress.incr_percent(0.25);
    /// assert_eq!(progress.percent(), 0.25);
    ///
    /// // Add another 30%
    /// let cmd2 = progress.incr_percent(0.3);
    /// assert_eq!(progress.percent(), 0.55);
    ///
    /// // Try to go over 100% (will be clamped)
    /// let cmd3 = progress.incr_percent(0.7);
    /// assert_eq!(progress.percent(), 1.0);
    /// ```
    ///
    /// ## Use in Loading Scenarios
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let mut download_progress = new(&[]);
    ///
    /// // Simulate incremental progress updates
    /// for _chunk in 0..10 {
    ///     let cmd = download_progress.incr_percent(0.1); // Add 10% each chunk
    ///     // Return cmd from your update() method
    /// }
    /// ```
    pub fn incr_percent(&mut self, v: f64) -> Cmd {
        self.set_percent(self.percent() + v)
    }

    /// Decreases the progress by a specified amount and returns an animation command.
    ///
    /// This is a convenience method that subtracts the given value from the current
    /// progress percentage. The result is automatically clamped to the valid
    /// range (0.0-1.0) and animated smoothly to the new position.
    ///
    /// # Arguments
    ///
    /// * `v` - Amount to subtract from current progress (positive values decrease progress)
    ///
    /// # Returns
    ///
    /// A `Cmd` that drives the animation to the new progress value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let mut progress = new(&[]);
    ///
    /// // Start at 100%
    /// progress.set_percent(1.0);
    ///
    /// // Decrease by 20%
    /// let cmd1 = progress.decr_percent(0.2);
    /// assert_eq!(progress.percent(), 0.8);
    ///
    /// // Decrease by another 30%
    /// let cmd2 = progress.decr_percent(0.3);
    /// assert_eq!(progress.percent(), 0.5);
    ///
    /// // Try to go below 0% (will be clamped)
    /// let cmd3 = progress.decr_percent(0.8);
    /// assert_eq!(progress.percent(), 0.0);
    /// ```
    ///
    /// ## Use in Error Recovery
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let mut upload_progress = new(&[]);
    /// upload_progress.set_percent(0.7); // 70% uploaded
    ///
    /// // Network error - need to retry from earlier point
    /// let cmd = upload_progress.decr_percent(0.2); // Back to 50%
    /// ```
    pub fn decr_percent(&mut self, v: f64) -> Cmd {
        self.set_percent(self.percent() - v)
    }

    /// Processes animation messages and updates the visual progress state.
    ///
    /// This method handles `FrameMsg` instances that drive the smooth animation
    /// between progress values. It should be called from your application's
    /// `update()` method to enable animated progress transitions.
    ///
    /// The method uses spring physics to gradually move the visual progress
    /// from its current position toward the target percentage, creating
    /// natural-looking animations.
    ///
    /// # Arguments
    ///
    /// * `msg` - A message that might contain animation frame data
    ///
    /// # Returns
    ///
    /// - `Some(Cmd)` if animation should continue (return this from your update method)
    /// - `None` if the message wasn't relevant or animation has finished
    ///
    /// # Examples
    ///
    /// ## Integration with bubbletea-rs
    /// ```rust
    /// use bubbles_rs::progress;
    /// use bubbletea_rs::{Model, Msg, Cmd};
    ///
    /// struct App {
    ///     progress: progress::Model,
    /// }
    ///
    /// impl Model for App {
    ///     fn update(&mut self, msg: Msg) -> Option<Cmd> {
    ///         // Always forward messages to progress bar first
    ///         if let Some(cmd) = self.progress.update(msg) {
    ///             return Some(cmd);
    ///         }
    ///
    ///         // Handle your own application messages
    ///         None
    ///     }
    /// #   fn init() -> (Self, Option<Cmd>) { (Self { progress: progress::new(&[]) }, None) }
    /// #   fn view(&self) -> String { String::new() }
    /// }
    /// ```
    ///
    /// ## Manual Animation Handling
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let mut progress = new(&[]);
    ///
    /// // Start an animation
    /// let initial_cmd = progress.set_percent(0.8);
    ///
    /// // In a real app, you'd handle this through bubbletea-rs
    /// // but here's what happens internally:
    /// // 1. FrameMsg is sent after a delay
    /// // 2. update() processes it and returns next FrameMsg
    /// // 3. Process continues until animation completes
    /// ```
    pub fn update(&mut self, msg: Msg) -> std::option::Option<Cmd> {
        if let Some(frame_msg) = msg.downcast_ref::<FrameMsg>() {
            if frame_msg.id != self.id || frame_msg.tag != self.tag {
                return std::option::Option::None;
            }

            // If we've more or less reached equilibrium, stop updating.
            if !self.is_animating() {
                return std::option::Option::None;
            }

            let (new_percent, new_velocity) =
                self.spring
                    .update(self.percent_shown, self.velocity, self.target_percent);
            self.percent_shown = new_percent;
            self.velocity = new_velocity;

            return std::option::Option::Some(self.next_frame());
        }

        std::option::Option::None
    }

    /// Renders the progress bar in its current animated state.
    ///
    /// This method displays the progress bar with the current visual progress,
    /// which may be different from the target percentage during animations.
    /// The output includes both the visual bar and percentage text (if enabled).
    ///
    /// For static rendering with a specific percentage, use `view_as()` instead.
    ///
    /// # Returns
    ///
    /// A formatted string containing the styled progress bar ready for terminal display.
    ///
    /// # Examples
    ///
    /// ## Basic Rendering
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let progress = new(&[]);
    /// let output = progress.view();
    /// println!("{}", output); // Displays: [░░░░░░░] 0%
    /// ```
    ///
    /// ## Animated Progress
    /// ```rust
    /// use bubbles_rs::progress::{new, with_width};
    ///
    /// let mut progress = new(&[with_width(20)]);
    ///
    /// // Set target percentage
    /// let cmd = progress.set_percent(0.6);
    ///
    /// // During animation, view() shows the current animated position
    /// let frame1 = progress.view(); // Might show: [██████░░░] 35%
    ///
    /// // After animation completes:
    /// let final_frame = progress.view(); // Shows: [███████████░] 60%
    /// ```
    ///
    /// ## Integration Example
    /// ```rust
    /// use bubbles_rs::progress::{new, with_solid_fill};
    ///
    /// let progress = new(&[
    ///     with_solid_fill("#2ecc71".to_string()),
    /// ]);
    ///
    /// // Use in your application's view method
    /// fn render_ui(progress: &bubbles_rs::progress::Model) -> String {
    ///     format!("Download Progress:\n{}\n", progress.view())
    /// }
    /// ```
    pub fn view(&self) -> String {
        self.view_as(self.percent_shown)
    }

    /// Renders the progress bar with a specific percentage value.
    ///
    /// This method bypasses the animation system and renders the progress bar
    /// at exactly the specified percentage. This is useful for static displays,
    /// testing, or when you want to show progress without animation.
    ///
    /// # Arguments
    ///
    /// * `percent` - Progress percentage as a float between 0.0 and 1.0
    ///
    /// # Returns
    ///
    /// A formatted string containing the styled progress bar at the exact percentage.
    ///
    /// # Examples
    ///
    /// ## Static Progress Display
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let progress = new(&[]);
    ///
    /// // Show various progress levels without animation
    /// let empty = progress.view_as(0.0);    // [░░░░░░░] 0%
    /// let half = progress.view_as(0.5);     // [███░░░░] 50%
    /// let full = progress.view_as(1.0);     // [███████] 100%
    /// ```
    ///
    /// ## Testing Different Progress Values
    /// ```rust
    /// use bubbles_rs::progress::{new, with_width, without_percentage};
    ///
    /// let progress = new(&[
    ///     with_width(10),
    ///     without_percentage(),
    /// ]);
    ///
    /// // Test various progress levels
    /// assert_eq!(progress.view_as(0.0).contains('█'), false);
    /// assert_eq!(progress.view_as(1.0).contains('░'), false);
    /// ```
    ///
    /// ## Dynamic Progress Calculation
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let progress = new(&[]);
    ///
    /// // Show progress based on calculation
    /// let completed_items = 7;
    /// let total_items = 10;
    /// let percentage = completed_items as f64 / total_items as f64;
    ///
    /// let view = progress.view_as(percentage);
    /// println!("Tasks: {}", view); // Shows 70% progress
    /// ```
    pub fn view_as(&self, percent: f64) -> String {
        let percent_view = self.percentage_view(percent);
        // Use visible width (ignoring ANSI escape sequences and wide chars)
        let percent_width = lipgloss::width_visible(&percent_view) as i32;
        let bar_view = self.bar_view(percent, percent_width);

        format!("{}{}", bar_view, percent_view)
    }

    /// Returns whether the progress bar is currently animating.
    ///
    /// This method checks if the progress bar is in the middle of an animated
    /// transition between progress values. It returns `false` when the visual
    /// progress has reached equilibrium with the target percentage.
    ///
    /// # Returns
    ///
    /// - `true` if the progress bar is currently animating
    /// - `false` if the animation has completed or no animation is in progress
    ///
    /// # Examples
    ///
    /// ## Checking Animation State
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let mut progress = new(&[]);
    ///
    /// // Initially not animating
    /// assert!(!progress.is_animating());
    ///
    /// // Start an animation
    /// let cmd = progress.set_percent(0.8);
    /// assert!(progress.is_animating()); // Now animating
    ///
    /// // After animation completes (in a real app, this happens via update())
    /// // assert!(!progress.is_animating());
    /// ```
    ///
    /// ## Conditional Rendering
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// let progress = new(&[]);
    ///
    /// // Show different UI based on animation state
    /// if progress.is_animating() {
    ///     println!("Progress updating...");
    /// } else {
    ///     println!("Progress stable at {}%", (progress.percent() * 100.0) as i32);
    /// }
    /// ```
    ///
    /// ## Performance Optimization
    /// ```rust
    /// use bubbles_rs::progress::new;
    ///
    /// fn should_update_ui(progress: &bubbles_rs::progress::Model) -> bool {
    ///     // Only redraw UI if progress is changing
    ///     progress.is_animating()
    /// }
    /// ```
    pub fn is_animating(&self) -> bool {
        let dist = (self.percent_shown - self.target_percent).abs();
        // Match Go logic: stop when close to equilibrium and velocity is low
        !(dist < 0.001 && self.velocity < 0.01)
    }

    /// Internal method to create next frame command
    fn next_frame(&self) -> Cmd {
        let id = self.id;
        let tag = self.tag;
        let duration = Duration::from_nanos(1_000_000_000 / FPS as u64);

        bubbletea_tick(duration, move |_| Box::new(FrameMsg { id, tag }) as Msg)
    }

    /// Internal method to render the progress bar
    fn bar_view(&self, percent: f64, text_width: i32) -> String {
        let tw = std::cmp::max(0, self.width - text_width); // total width
        let fw = std::cmp::max(0, std::cmp::min(tw, ((tw as f64) * percent).round() as i32)); // filled width

        let mut result = String::new();

        if self.use_ramp {
            // Proper gradient fill using perceptual blending via lipgloss
            let total_width_for_gradient = if self.scale_ramp { fw } else { tw };
            let grad_len = std::cmp::max(2, total_width_for_gradient) as usize;

            let start = LGColor::from(self.ramp_color_a.as_str());
            let end = LGColor::from(self.ramp_color_b.as_str());
            let gradient_colors = blend_1d(grad_len, vec![start, end]);

            if fw == 1 {
                // Choose middle of the gradient for width=1, matching Go's 0.5 choice
                let mid_idx = (grad_len as f64 * 0.5).floor() as usize;
                let mid_idx = std::cmp::min(mid_idx, grad_len - 1);
                let styled = Style::new()
                    .foreground(gradient_colors[mid_idx].clone())
                    .render(&self.full.to_string());
                result.push_str(&styled);
            } else {
                for i in 0..fw as usize {
                    let idx = i; // gradient indexed from left
                    let color_idx = std::cmp::min(idx, grad_len - 1);
                    let styled = Style::new()
                        .foreground(gradient_colors[color_idx].clone())
                        .render(&self.full.to_string());
                    result.push_str(&styled);
                }
            }
        } else {
            // Solid fill
            let styled = Style::new()
                .foreground(lipgloss::Color::from(self.full_color.as_str()))
                .render(&self.full.to_string());
            result.push_str(&styled.repeat(fw as usize));
        }

        // Empty fill
        let empty_styled = Style::new()
            .foreground(lipgloss::Color::from(self.empty_color.as_str()))
            .render(&self.empty.to_string());
        let n = std::cmp::max(0, tw - fw);
        result.push_str(&empty_styled.repeat(n as usize));

        result
    }

    /// Internal method to render percentage view
    fn percentage_view(&self, percent: f64) -> String {
        if !self.show_percentage {
            return String::new();
        }

        let percent = percent.clamp(0.0, 1.0);
        let percentage = format!(" {:3.0}%", percent * 100.0); // Simplified format
        self.percentage_style.render(&percentage)
    }

    /// Internal method to set gradient colors
    fn set_ramp(&mut self, color_a: String, color_b: String, scaled: bool) {
        self.use_ramp = true;
        self.scale_ramp = scaled;
        self.ramp_color_a = color_a;
        self.ramp_color_b = color_b;
    }
}

impl BubbleTeaModel for Model {
    fn init() -> (Self, std::option::Option<Cmd>) {
        let model = new(&[]);
        (model, std::option::Option::None)
    }

    fn update(&mut self, msg: Msg) -> std::option::Option<Cmd> {
        self.update(msg)
    }

    fn view(&self) -> String {
        self.view()
    }
}

impl Default for Model {
    fn default() -> Self {
        new(&[])
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::progress::{
        new, new_model, with_default_gradient, with_fill_characters, with_gradient,
        with_solid_fill, with_spring_options, with_width, without_percentage, FrameMsg,
    };

    #[test]
    fn test_new_with_no_options() {
        // Test Go's: New()
        let progress = new(&[]);

        assert_eq!(progress.width, DEFAULT_WIDTH);
        assert_eq!(progress.full, '█');
        assert_eq!(progress.empty, '░');
        assert_eq!(progress.full_color, "#7571F9");
        assert_eq!(progress.empty_color, "#606060");
        assert!(progress.show_percentage);
        assert_eq!(progress.percent_format, " %3.0f%%");
        assert!(!progress.use_ramp);
        assert_eq!(progress.percent(), 0.0);
    }

    #[test]
    fn test_new_with_width() {
        // Test Go's: New(WithWidth(60))
        let progress = new(&[with_width(60)]);
        assert_eq!(progress.width, 60);
    }

    #[test]
    fn test_new_with_solid_fill() {
        // Test Go's: New(WithSolidFill("#ff0000"))
        let progress = new(&[with_solid_fill("#ff0000".to_string())]);
        assert_eq!(progress.full_color, "#ff0000");
        assert!(!progress.use_ramp);
    }

    #[test]
    fn test_new_with_fill_characters() {
        // Test Go's: New(WithFillCharacters('▓', '▒'))
        let progress = new(&[with_fill_characters('▓', '▒')]);
        assert_eq!(progress.full, '▓');
        assert_eq!(progress.empty, '▒');
    }

    #[test]
    fn test_new_without_percentage() {
        // Test Go's: New(WithoutPercentage())
        let progress = new(&[without_percentage()]);
        assert!(!progress.show_percentage);
    }

    #[test]
    fn test_new_with_gradient() {
        // Test Go's: New(WithGradient("#ff0000", "#0000ff"))
        let progress = new(&[with_gradient("#ff0000".to_string(), "#0000ff".to_string())]);
        assert!(progress.use_ramp);
        assert_eq!(progress.ramp_color_a, "#ff0000");
        assert_eq!(progress.ramp_color_b, "#0000ff");
        assert!(!progress.scale_ramp);
    }

    #[test]
    fn test_new_with_default_gradient() {
        // Test Go's: New(WithDefaultGradient())
        let progress = new(&[with_default_gradient()]);
        assert!(progress.use_ramp);
        assert_eq!(progress.ramp_color_a, "#5A56E0");
        assert_eq!(progress.ramp_color_b, "#EE6FF8");
    }

    #[test]
    fn test_new_with_spring_options() {
        // Test Go's: New(WithSpringOptions(20.0, 0.8))
        let progress = new(&[with_spring_options(20.0, 0.8)]);
        assert!(progress.spring_customized);
        assert_eq!(progress.spring.frequency, 20.0);
        assert_eq!(progress.spring.damping, 0.8);
    }

    #[test]
    fn test_new_with_multiple_options() {
        // Test Go's: New(WithWidth(80), WithSolidFill("#00ff00"), WithoutPercentage())
        let progress = new(&[
            with_width(80),
            with_solid_fill("#00ff00".to_string()),
            without_percentage(),
        ]);

        assert_eq!(progress.width, 80);
        assert_eq!(progress.full_color, "#00ff00");
        assert!(!progress.show_percentage);
        assert!(!progress.use_ramp);
    }

    #[test]
    fn test_deprecated_new_model() {
        // Test Go's deprecated: NewModel()
        #[allow(deprecated)]
        let progress = new_model(&[with_width(50)]);
        assert_eq!(progress.width, 50);
    }

    #[test]
    fn test_percent_method() {
        // Test Go's: Percent() float64
        let mut progress = new(&[]);
        assert_eq!(progress.percent(), 0.0);

        std::mem::drop(progress.set_percent(0.75));
        assert_eq!(progress.percent(), 0.75);
    }

    #[test]
    fn test_set_percent() {
        // Test Go's: SetPercent(p float64) tea.Cmd
        let mut progress = new(&[]);

        // Should clamp values
        std::mem::drop(progress.set_percent(1.5)); // Over 1.0
        assert_eq!(progress.percent(), 1.0);

        std::mem::drop(progress.set_percent(-0.5)); // Under 0.0
        assert_eq!(progress.percent(), 0.0);

        std::mem::drop(progress.set_percent(0.5)); // Normal value
        assert_eq!(progress.percent(), 0.5);

        // Should increment tag
        let original_tag = progress.tag;
        std::mem::drop(progress.set_percent(0.6));
        assert_eq!(progress.tag, original_tag + 1);
    }

    #[test]
    fn test_incr_percent() {
        // Test Go's: IncrPercent(v float64) tea.Cmd
        let mut progress = new(&[]);
        std::mem::drop(progress.set_percent(0.3));

        std::mem::drop(progress.incr_percent(0.2));
        assert_eq!(progress.percent(), 0.5);

        // Should clamp at 1.0
        std::mem::drop(progress.incr_percent(0.8));
        assert_eq!(progress.percent(), 1.0);
    }

    #[test]
    fn test_decr_percent() {
        // Test Go's: DecrPercent(v float64) tea.Cmd
        let mut progress = new(&[]);
        std::mem::drop(progress.set_percent(0.7));

        std::mem::drop(progress.decr_percent(0.2));
        assert!((progress.percent() - 0.5).abs() < 1e-9);

        // Should clamp at 0.0
        std::mem::drop(progress.decr_percent(0.8));
        assert_eq!(progress.percent(), 0.0);
    }

    #[test]
    fn test_set_spring_options() {
        // Test Go's: SetSpringOptions(frequency, damping float64)
        let mut progress = new(&[]);
        progress.set_spring_options(25.0, 1.5);

        assert_eq!(progress.spring.frequency, 25.0);
        assert_eq!(progress.spring.damping, 1.5);
        assert_eq!(progress.spring.fps, FPS as f64);
    }

    #[test]
    fn test_is_animating() {
        // Test Go's: IsAnimating() bool
        let mut progress = new(&[]);

        // Initially not animating (at equilibrium)
        assert!(!progress.is_animating());

        // After setting target, should be animating if there's a difference
        std::mem::drop(progress.set_percent(0.5));
        // Since percent_shown is still 0.0 and target is 0.5, should be animating
        assert!(progress.is_animating());

        // When at equilibrium, should not be animating
        progress.percent_shown = 0.5;
        progress.velocity = 0.0;
        assert!(!progress.is_animating());
    }

    #[test]
    fn test_update_with_frame_msg() {
        // Test Go's: Update with FrameMsg
        let mut progress = new(&[]);
        std::mem::drop(progress.set_percent(0.5)); // Set target

        let frame_msg = FrameMsg {
            id: progress.id,
            tag: progress.tag,
        };

        let result = progress.update(Box::new(frame_msg));
        assert!(result.is_some()); // Should return next frame command if animating
    }

    #[test]
    fn test_update_with_wrong_id() {
        // Test that progress rejects FrameMsg with wrong ID
        let mut progress = new(&[]);

        let wrong_frame = FrameMsg {
            id: progress.id + 999, // Wrong ID
            tag: progress.tag,
        };

        let result = progress.update(Box::new(wrong_frame));
        assert!(result.is_none()); // Should reject
    }

    #[test]
    fn test_update_with_wrong_tag() {
        // Test that progress rejects FrameMsg with wrong tag
        let mut progress = new(&[]);

        let wrong_frame = FrameMsg {
            id: progress.id,
            tag: progress.tag + 999, // Wrong tag
        };

        let result = progress.update(Box::new(wrong_frame));
        assert!(result.is_none()); // Should reject
    }

    #[test]
    fn test_view_basic() {
        // Test Go's: View() string
        let progress = new(&[with_width(10)]);
        let view = progress.view();

        // Should contain progress bar characters
        assert!(view.contains('░')); // Empty char
                                     // At 0%, should be mostly empty
        let empty_count = view.chars().filter(|&c| c == '░').count();
        assert!(empty_count > 0);
    }

    #[test]
    fn test_view_as() {
        // Test Go's: ViewAs(percent float64) string
        let progress = new(&[with_width(10)]);

        // Test with specific percentage
        let view_50 = progress.view_as(0.5);
        let view_100 = progress.view_as(1.0);

        // 100% should have more filled chars than 50%
        let filled_50 = view_50.chars().filter(|&c| c == '█').count();
        let filled_100 = view_100.chars().filter(|&c| c == '█').count();
        assert!(filled_100 > filled_50);
    }

    #[test]
    fn test_view_without_percentage() {
        // Test view without percentage display
        let progress = new(&[without_percentage(), with_width(10)]);
        let view = progress.view_as(0.5);

        // Should not contain percentage text
        assert!(!view.contains('%'));
    }

    #[test]
    fn test_view_with_percentage() {
        // Test view with percentage display
        let progress = new(&[with_width(10)]); // Default includes percentage
        let view = progress.view_as(0.75);

        // Should contain percentage text
        assert!(view.contains('%'));
        assert!(view.contains("75")); // 75%
    }

    #[test]
    fn test_spring_animation_physics() {
        // Test that spring physics work correctly
        let spring = Spring::new(60.0, 10.0, 1.0);

        // Test movement towards target
        let (new_pos, _new_vel) = spring.update(0.0, 0.0, 1.0);

        // Should move towards target (1.0) from 0.0
        assert!(new_pos > 0.0);
        assert!(new_pos < 1.0); // Shouldn't overshoot immediately
    }

    #[test]
    fn test_bar_view_width_calculation() {
        // Test that bar width calculations match Go logic
        let progress = new(&[with_width(20), without_percentage()]);

        let view_0 = progress.view_as(0.0); // 0%
        let view_50 = progress.view_as(0.5); // 50%
        let view_100 = progress.view_as(1.0); // 100%

        // All views should have same total length based on visible characters
        assert_eq!(lipgloss::width_visible(&view_0), 20);
        assert_eq!(lipgloss::width_visible(&view_50), 20);
        assert_eq!(lipgloss::width_visible(&view_100), 20);

        // 0% should be all empty, 100% should be all full
        let bar_0 = progress.bar_view(0.0, 0);
        let bar_100 = progress.bar_view(1.0, 0);
        let bar_0_clean = lipgloss::strip_ansi(&bar_0);
        let bar_100_clean = lipgloss::strip_ansi(&bar_100);
        assert!(bar_0_clean.chars().all(|c| c == '░' || c.is_whitespace()));
        assert!(bar_100_clean.chars().all(|c| c == '█' || c.is_whitespace()));
    }

    #[test]
    fn test_gradient_vs_solid_fill() {
        // Test difference between gradient and solid fill
        let solid = new(&[
            with_solid_fill("#ff0000".to_string()),
            with_width(10),
            without_percentage(),
        ]);
        let gradient = new(&[
            with_gradient("#ff0000".to_string(), "#00ff00".to_string()),
            with_width(10),
            without_percentage(),
        ]);

        assert!(!solid.use_ramp);
        assert!(gradient.use_ramp);

        // Both should render something at 50%
        let solid_view = solid.view_as(0.5);
        let gradient_view = gradient.view_as(0.5);

        assert!(!solid_view.is_empty());
        assert!(!gradient_view.is_empty());
    }

    #[test]
    fn test_gradient_first_last_colors_match() {
        // Skip when colors are disabled in the environment (common in CI)
        if std::env::var("NO_COLOR").is_ok() || std::env::var("NOCOLOR").is_ok() {
            return;
        }
        // Parity with Go's TestGradient: first and last color at 100% should match endpoints
        const RESET: &str = "\x1b[0m";
        let col_a = "#FF0000";
        let col_b = "#00FF00";

        for scale in [false, true] {
            for &w in &[3, 5, 50] {
                let mut opts = vec![without_percentage(), with_width(w)];
                if scale {
                    opts.push(with_scaled_gradient(col_a.to_string(), col_b.to_string()));
                } else {
                    opts.push(with_gradient(col_a.to_string(), col_b.to_string()));
                }

                let p = new(&opts);
                let res = p.view_as(1.0);

                // Extract color sequences by splitting at Full + RESET
                let splitter = format!("{}{}", p.full, RESET);
                let mut colors: Vec<&str> = res.split(&splitter).collect();
                if !colors.is_empty() {
                    // Discard last empty part after the final split
                    colors.pop();
                }

                // Build expected first/last color sequences using style rendering
                let expected_first_full = lipgloss::Style::new()
                    .foreground(lipgloss::Color::from(col_a))
                    .render(&p.full.to_string());
                let expected_last_full = lipgloss::Style::new()
                    .foreground(lipgloss::Color::from(col_b))
                    .render(&p.full.to_string());

                let exp_first = expected_first_full
                    .split(&format!("{}{}", p.full, RESET))
                    .next()
                    .unwrap_or("");
                let exp_last = expected_last_full
                    .split(&format!("{}{}", p.full, RESET))
                    .next()
                    .unwrap_or("");

                // Sanity: need at least width items
                assert!(colors.len() >= (w as usize).saturating_sub(0));

                // Compare first and last color control sequences
                let first_color = colors.first().copied().unwrap_or("");
                let last_color = colors.last().copied().unwrap_or("");
                assert_eq!(
                    exp_first, first_color,
                    "first gradient color should match start"
                );
                assert_eq!(exp_last, last_color, "last gradient color should match end");
            }
        }
    }

    #[test]
    fn test_unique_ids() {
        // Test that multiple progress bars get unique IDs
        let progress1 = new(&[]);
        let progress2 = new(&[]);

        assert_ne!(progress1.id, progress2.id);
    }

    #[test]
    fn test_default_implementation() {
        // Test Default trait implementation
        let progress = Model::default();
        assert_eq!(progress.width, DEFAULT_WIDTH);
        assert_eq!(progress.percent(), 0.0);
    }
}
