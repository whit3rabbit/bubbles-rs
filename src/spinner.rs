//! Spinner component for Bubble Tea applications.
//!
//! This module provides a spinner component for Bubble Tea applications.
//! It closely matches the API of the Go bubbles spinner component for 1-1 compatibility.
//!
//! # Basic Usage
//!
//! ```rust
//! use bubbletea_widgets::spinner::{new, with_spinner, with_style, DOT};
//! use lipgloss_extras::prelude::*;
//!
//! // Create a spinner with default settings
//! let spinner = new(&[]);
//!
//! // Create a spinner with custom settings using the option pattern
//! let spinner = new(&[
//!     with_spinner(DOT.clone()),
//!     with_style(Style::new().foreground(lipgloss::Color::from("red"))),
//! ]);
//! ```
//!
//! # Available Spinners
//!
//! The following predefined spinners are available as constants (matching Go exactly):
//! - `LINE`: Basic line spinner (|, /, -, \)
//! - `DOT`: Braille dot pattern spinner
//! - `MINI_DOT`: Smaller braille dot pattern
//! - `JUMP`: Jumping dot animation
//! - `PULSE`: Block fade animation (█, ▓, ▒, ░)
//! - `POINTS`: Three dot bounce animation
//! - `GLOBE`: Earth emoji rotation
//! - `MOON`: Moon phase animation
//! - `MONKEY`: See-no-evil monkey sequence
//! - `METER`: Progress bar style animation
//! - `HAMBURGER`: Trigram symbol animation
//! - `ELLIPSIS`: Text ellipsis animation ("", ".", "..", "...")
//!
//! # bubbletea-rs Integration
//!
//! ```rust
//! use bubbletea_rs::{Model as BubbleTeaModel, Msg, Cmd};
//! use bubbletea_widgets::spinner::{new, with_spinner, DOT, TickMsg};
//!
//! struct MyApp {
//!     spinner: bubbletea_widgets::spinner::Model,
//! }
//!
//! impl BubbleTeaModel for MyApp {
//!     fn init() -> (Self, Option<Cmd>) {
//!         let spinner = new(&[with_spinner(DOT.clone())]);
//!         // Spinners start automatically when created
//!         (Self { spinner }, None)
//!     }
//!
//!     fn update(&mut self, msg: Msg) -> Option<Cmd> {
//!         // Forward spinner messages to spinner
//!         self.spinner.update(msg)
//!     }
//!
//!     fn view(&self) -> String {
//!         format!("{} Loading...", self.spinner.view())
//!     }
//! }
//! ```

use bubbletea_rs::{tick as bubbletea_tick, Cmd, Model as BubbleTeaModel, Msg};
use lipgloss_extras::prelude::*;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

// Internal ID management for spinner instances
static LAST_ID: AtomicI64 = AtomicI64::new(0);

/// Generates the next unique ID for spinner instances.
///
/// This function is thread-safe and ensures that each spinner instance
/// receives a unique identifier for message routing.
///
/// # Returns
///
/// Returns a unique positive integer ID.
fn next_id() -> i64 {
    LAST_ID.fetch_add(1, Ordering::SeqCst) + 1
}

/// Spinner configuration defining animation frames and timing.
///
/// A Spinner contains the visual frames and frame rate for a terminal spinner animation.
/// This matches the Go bubbles Spinner struct for full API compatibility.
///
/// # Fields
///
/// * `frames` - Vector of strings representing each animation frame
/// * `fps` - Duration between frame updates (smaller = faster animation)
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::spinner::Spinner;
/// use std::time::Duration;
///
/// // Create a custom spinner
/// let custom = Spinner::new(
///     vec!["◐".to_string(), "◓".to_string(), "◑".to_string(), "◒".to_string()],
///     Duration::from_millis(200)
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Spinner {
    /// Animation frames to cycle through.
    pub frames: Vec<String>,
    /// Delay between frames; smaller is faster.
    pub fps: Duration,
}

impl Spinner {
    /// Creates a new Spinner with the given frames and timing.
    ///
    /// # Arguments
    ///
    /// * `frames` - Vector of strings representing each animation frame
    /// * `fps` - Duration between frame updates
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::spinner::Spinner;
    /// use std::time::Duration;
    ///
    /// let spinner = Spinner::new(
    ///     vec!["|".to_string(), "/".to_string(), "-".to_string(), "\\".to_string()],
    ///     Duration::from_millis(100)
    /// );
    /// assert_eq!(spinner.frames.len(), 4);
    /// ```
    pub fn new(frames: Vec<String>, fps: Duration) -> Self {
        Self { frames, fps }
    }
}

// Predefined spinner styles matching the Go implementation exactly

/// Line spinner - matches Go's Line constant
pub static LINE: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "|".to_string(),
        "/".to_string(),
        "-".to_string(),
        "\\".to_string(),
    ],
    fps: Duration::from_millis(100), // time.Second / 10
});

/// Dot spinner - matches Go's Dot constant  
pub static DOT: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "⣾ ".to_string(),
        "⣽ ".to_string(),
        "⣻ ".to_string(),
        "⢿ ".to_string(),
        "⡿ ".to_string(),
        "⣟ ".to_string(),
        "⣯ ".to_string(),
        "⣷ ".to_string(),
    ],
    fps: Duration::from_millis(100), // time.Second / 10
});

/// MiniDot spinner - matches Go's MiniDot constant
pub static MINI_DOT: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "⠋".to_string(),
        "⠙".to_string(),
        "⠹".to_string(),
        "⠸".to_string(),
        "⠼".to_string(),
        "⠴".to_string(),
        "⠦".to_string(),
        "⠧".to_string(),
        "⠇".to_string(),
        "⠏".to_string(),
    ],
    fps: Duration::from_millis(83), // time.Second / 12
});

/// Jump spinner - matches Go's Jump constant
pub static JUMP: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "⢄".to_string(),
        "⢂".to_string(),
        "⢁".to_string(),
        "⡁".to_string(),
        "⡈".to_string(),
        "⡐".to_string(),
        "⡠".to_string(),
    ],
    fps: Duration::from_millis(100), // time.Second / 10
});

/// Pulse spinner - matches Go's Pulse constant
pub static PULSE: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "█".to_string(),
        "▓".to_string(),
        "▒".to_string(),
        "░".to_string(),
    ],
    fps: Duration::from_millis(125), // time.Second / 8
});

/// Points spinner - matches Go's Points constant
pub static POINTS: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "∙∙∙".to_string(),
        "●∙∙".to_string(),
        "∙●∙".to_string(),
        "∙∙●".to_string(),
    ],
    fps: Duration::from_millis(143), // time.Second / 7 (approximately)
});

/// Globe spinner - matches Go's Globe constant
pub static GLOBE: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec!["🌍".to_string(), "🌎".to_string(), "🌏".to_string()],
    fps: Duration::from_millis(250), // time.Second / 4
});

/// Moon spinner - matches Go's Moon constant
pub static MOON: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "🌑".to_string(),
        "🌒".to_string(),
        "🌓".to_string(),
        "🌔".to_string(),
        "🌕".to_string(),
        "🌖".to_string(),
        "🌗".to_string(),
        "🌘".to_string(),
    ],
    fps: Duration::from_millis(125), // time.Second / 8
});

/// Monkey spinner - matches Go's Monkey constant
pub static MONKEY: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec!["🙈".to_string(), "🙉".to_string(), "🙊".to_string()],
    fps: Duration::from_millis(333), // time.Second / 3
});

/// Meter spinner - matches Go's Meter constant
pub static METER: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "▱▱▱".to_string(),
        "▰▱▱".to_string(),
        "▰▰▱".to_string(),
        "▰▰▰".to_string(),
        "▰▰▱".to_string(),
        "▰▱▱".to_string(),
        "▱▱▱".to_string(),
    ],
    fps: Duration::from_millis(143), // time.Second / 7 (approximately)
});

/// Hamburger spinner - matches Go's Hamburger constant  
pub static HAMBURGER: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "☱".to_string(),
        "☲".to_string(),
        "☴".to_string(),
        "☲".to_string(),
    ],
    fps: Duration::from_millis(333), // time.Second / 3
});

/// Ellipsis spinner - matches Go's Ellipsis constant
pub static ELLIPSIS: Lazy<Spinner> = Lazy::new(|| Spinner {
    frames: vec![
        "".to_string(),
        ".".to_string(),
        "..".to_string(),
        "...".to_string(),
    ],
    fps: Duration::from_millis(333), // time.Second / 3
});

// Deprecated function aliases for backward compatibility
/// Deprecated: use the `LINE` constant instead.
#[deprecated(since = "0.0.7", note = "use LINE constant instead")]
pub fn line() -> Spinner {
    LINE.clone()
}

/// Deprecated: use the `DOT` constant instead.
#[deprecated(since = "0.0.7", note = "use DOT constant instead")]
pub fn dot() -> Spinner {
    DOT.clone()
}

/// Deprecated: use the `MINI_DOT` constant instead.
#[deprecated(since = "0.0.7", note = "use MINI_DOT constant instead")]
pub fn mini_dot() -> Spinner {
    MINI_DOT.clone()
}

/// Deprecated: use the `JUMP` constant instead.
#[deprecated(since = "0.0.7", note = "use JUMP constant instead")]
pub fn jump() -> Spinner {
    JUMP.clone()
}

/// Deprecated: use the `PULSE` constant instead.
#[deprecated(since = "0.0.7", note = "use PULSE constant instead")]
pub fn pulse() -> Spinner {
    PULSE.clone()
}

/// Deprecated: use the `POINTS` constant instead.
#[deprecated(since = "0.0.7", note = "use POINTS constant instead")]
pub fn points() -> Spinner {
    POINTS.clone()
}

/// Deprecated: use the `GLOBE` constant instead.
#[deprecated(since = "0.0.7", note = "use GLOBE constant instead")]
pub fn globe() -> Spinner {
    GLOBE.clone()
}

/// Deprecated: use the `MOON` constant instead.
#[deprecated(since = "0.0.7", note = "use MOON constant instead")]
pub fn moon() -> Spinner {
    MOON.clone()
}

/// Deprecated: use the `MONKEY` constant instead.
#[deprecated(since = "0.0.7", note = "use MONKEY constant instead")]
pub fn monkey() -> Spinner {
    MONKEY.clone()
}

/// Deprecated: use the `METER` constant instead.
#[deprecated(since = "0.0.7", note = "use METER constant instead")]
pub fn meter() -> Spinner {
    METER.clone()
}

/// Deprecated: use the `HAMBURGER` constant instead.
#[deprecated(since = "0.0.7", note = "use HAMBURGER constant instead")]
pub fn hamburger() -> Spinner {
    HAMBURGER.clone()
}

/// Deprecated: use the `ELLIPSIS` constant instead.
#[deprecated(since = "0.0.7", note = "use ELLIPSIS constant instead")]
pub fn ellipsis() -> Spinner {
    ELLIPSIS.clone()
}

/// Message indicating that the timer has ticked and the spinner should advance one frame.
///
/// TickMsg is used by the bubbletea-rs event system to trigger spinner animation updates.
/// Each message contains timing information and routing data to ensure proper message delivery.
/// This exactly matches the Go bubbles TickMsg struct for API compatibility.
///
/// # Fields
///
/// * `time` - Timestamp when the tick occurred
/// * `id` - Unique identifier of the target spinner (0 for global messages)
/// * `tag` - Internal sequence number to prevent message flooding
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::spinner::{new, DOT};
/// use bubbletea_widgets::spinner::{with_spinner};
///
/// let spinner = new(&[with_spinner(DOT.clone())]);
/// let tick_msg = spinner.tick_msg();
/// assert_eq!(tick_msg.id, spinner.id());
/// ```
#[derive(Debug, Clone)]
pub struct TickMsg {
    /// Time is the time at which the tick occurred.
    pub time: std::time::SystemTime,
    /// ID is the identifier of the spinner that this message belongs to.
    pub id: i64,
    /// tag is used internally to prevent spinner from receiving too many messages.
    tag: i64,
}

/// Model represents the state and configuration of a spinner component.
///
/// The Model struct contains all the state needed to render and animate a spinner,
/// including the animation frames, styling, current position, and unique identifier
/// for message routing. This matches the Go bubbles spinner.Model for full compatibility.
///
/// # Fields
///
/// * `spinner` - The Spinner configuration (frames and timing)
/// * `style` - Lipgloss Style for visual formatting
/// * `frame` - Current animation frame index (private)
/// * `id` - Unique instance identifier for message routing (private)
/// * `tag` - Message sequence number to prevent flooding (private)
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::spinner::{new, with_spinner, DOT};
/// use lipgloss_extras::prelude::*;
///
/// let mut spinner = new(&[
///     with_spinner(DOT.clone())
/// ]);
///
/// // Use in a bubbletea-rs application
/// let view = spinner.view(); // Returns current frame as a styled string
/// ```
#[derive(Debug)]
pub struct Model {
    /// Spinner settings to use.
    pub spinner: Spinner,
    /// Style sets the styling for the spinner.
    pub style: Style,
    frame: usize,
    id: i64,
    tag: i64,
}

/// Configuration option for creating a new spinner with custom settings.
///
/// SpinnerOption implements the options pattern used by the `new()` function
/// to configure spinner instances. This matches Go's functional options pattern
/// used in the original bubbles library.
///
/// # Variants
///
/// * `WithSpinner(Spinner)` - Sets the animation frames and timing
/// * `WithStyle(Style)` - Sets the lipgloss styling
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::spinner::{new, with_spinner, with_style, DOT};
/// use lipgloss_extras::prelude::*;
///
/// let spinner = new(&[
///     with_spinner(DOT.clone()),
///     with_style(Style::new().foreground(Color::from("red")))
/// ]);
/// ```
pub enum SpinnerOption {
    /// Sets the animation frames and timing to use.
    WithSpinner(Spinner),
    /// Sets the lipgloss style for rendering the spinner.
    WithStyle(Box<Style>),
}

impl SpinnerOption {
    fn apply(&self, m: &mut Model) {
        match self {
            SpinnerOption::WithSpinner(spinner) => m.spinner = spinner.clone(),
            SpinnerOption::WithStyle(style) => m.style = style.as_ref().clone(),
        }
    }
}

/// Creates a SpinnerOption to set the animation frames and timing.
///
/// This function creates an option that can be passed to `new()` to configure
/// the spinner's animation. Matches Go's WithSpinner function for API compatibility.
///
/// # Arguments
///
/// * `spinner` - The Spinner configuration to use
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::spinner::{new, with_spinner, DOT};
///
/// let spinner_model = new(&[with_spinner(DOT.clone())]);
/// assert_eq!(spinner_model.spinner.frames.len(), 8); // DOT has 8 frames
/// ```
pub fn with_spinner(spinner: Spinner) -> SpinnerOption {
    SpinnerOption::WithSpinner(spinner)
}

/// Creates a SpinnerOption to set the visual styling.
///
/// This function creates an option that can be passed to `new()` to configure
/// the spinner's appearance using lipgloss styling. Matches Go's WithStyle function.
///
/// # Arguments
///
/// * `style` - The lipgloss Style to apply to the spinner
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::spinner::{new, with_style};
/// use lipgloss_extras::prelude::*;
///
/// let red_style = Style::new().foreground(Color::from("red"));
/// let spinner = new(&[with_style(red_style)]);
/// // Spinner will render in red color
/// ```
pub fn with_style(style: Style) -> SpinnerOption {
    SpinnerOption::WithStyle(Box::new(style))
}

impl Model {
    /// Creates a new spinner model with default settings.
    ///
    /// Creates a spinner using the LINE animation with default styling.
    /// Each spinner instance gets a unique ID for message routing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::spinner::Model;
    ///
    /// let spinner = Model::new();
    /// assert!(spinner.id() > 0);
    /// ```
    pub fn new() -> Self {
        Self {
            spinner: LINE.clone(),
            style: Style::new(),
            frame: 0,
            id: next_id(),
            tag: 0,
        }
    }

    /// Creates a new spinner model with custom configuration options.
    ///
    /// This function implements the options pattern to create a customized spinner.
    /// It matches Go's New function exactly for API compatibility.
    ///
    /// # Arguments
    ///
    /// * `opts` - Slice of SpinnerOption values to configure the spinner
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::spinner::{Model, with_spinner, DOT};
    ///
    /// let spinner = Model::new_with_options(&[with_spinner(DOT.clone())]);
    /// assert_eq!(spinner.spinner.frames.len(), 8);
    /// ```
    pub fn new_with_options(opts: &[SpinnerOption]) -> Self {
        let mut m = Self {
            spinner: LINE.clone(),
            style: Style::new(),
            frame: 0,
            id: next_id(),
            tag: 0,
        };

        for opt in opts {
            opt.apply(&mut m);
        }

        m
    }

    /// Sets the spinner animation configuration using builder pattern.
    ///
    /// # Arguments
    ///
    /// * `spinner` - The Spinner configuration to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::spinner::{Model, DOT};
    ///
    /// let spinner = Model::new().with_spinner(DOT.clone());
    /// assert_eq!(spinner.spinner.frames.len(), 8);
    /// ```
    pub fn with_spinner(mut self, spinner: Spinner) -> Self {
        self.spinner = spinner;
        self
    }

    /// Sets the visual styling using builder pattern.
    ///
    /// # Arguments
    ///
    /// * `style` - The lipgloss Style to apply
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::spinner::Model;
    /// use lipgloss_extras::prelude::*;
    ///
    /// let spinner = Model::new()
    ///     .with_style(Style::new().foreground(Color::from("blue")));
    /// ```
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Returns the spinner's unique identifier.
    ///
    /// Each spinner instance has a unique ID used for message routing
    /// to ensure tick messages are delivered to the correct spinner.
    /// Matches Go's ID() method for API compatibility.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::spinner::Model;
    ///
    /// let spinner1 = Model::new();
    /// let spinner2 = Model::new();
    /// assert_ne!(spinner1.id(), spinner2.id());
    /// ```
    pub fn id(&self) -> i64 {
        self.id
    }

    /// Creates a tick message to advance the spinner animation.
    ///
    /// This method creates a TickMsg that can be sent through the bubbletea-rs
    /// message system to trigger the next animation frame. The message includes
    /// the current time, spinner ID, and tag for proper routing.
    /// Matches Go's Tick() method for API compatibility.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::spinner::Model;
    ///
    /// let spinner = Model::new();
    /// let tick_msg = spinner.tick_msg();
    /// assert_eq!(tick_msg.id, spinner.id());
    /// ```
    pub fn tick_msg(&self) -> TickMsg {
        TickMsg {
            time: std::time::SystemTime::now(),
            id: self.id,
            tag: self.tag,
        }
    }

    /// Creates a bubbletea-rs command to schedule the next tick.
    ///
    /// This internal method creates a Cmd that will trigger after the spinner's
    /// frame duration, sending a TickMsg to continue the animation loop.
    ///
    /// # Returns
    ///
    /// Returns a Cmd that schedules the next animation frame update.
    fn tick(&self) -> Cmd {
        let id = self.id;
        let tag = self.tag;
        let fps = self.spinner.fps;

        bubbletea_tick(fps, move |_| {
            Box::new(TickMsg {
                time: std::time::SystemTime::now(),
                id,
                tag,
            }) as Msg
        })
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    /// Processes messages and updates the spinner state.
    ///
    /// This is the standard bubbletea-rs update function that processes incoming messages.
    /// It handles TickMsg messages to advance the animation and ignores other message types.
    /// The function includes ID and tag validation to ensure proper message routing and
    /// prevent animation rate issues. Matches Go's Update method exactly.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to process
    ///
    /// # Returns
    ///
    /// Returns Some(Cmd) to schedule the next tick, or None if the message was ignored.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::spinner::Model;
    ///
    /// let mut spinner = Model::new();
    /// let tick_msg = spinner.tick_msg();
    /// let cmd = spinner.update(Box::new(tick_msg));
    /// assert!(cmd.is_some()); // Should return next tick command
    /// ```
    pub fn update(&mut self, msg: Msg) -> std::option::Option<Cmd> {
        if let Some(tick_msg) = msg.downcast_ref::<TickMsg>() {
            // If an ID is set, and the ID doesn't belong to this spinner, reject the message.
            if tick_msg.id > 0 && tick_msg.id != self.id {
                return None;
            }

            // If a tag is set, and it's not the one we expect, reject the message.
            // This prevents the spinner from receiving too many messages and thus spinning too fast.
            if tick_msg.tag > 0 && tick_msg.tag != self.tag {
                return None;
            }

            self.frame += 1;
            if self.frame >= self.spinner.frames.len() {
                self.frame = 0;
            }

            self.tag += 1;
            return std::option::Option::Some(self.tick());
        }

        std::option::Option::None
    }

    /// Renders the current spinner frame as a styled string.
    ///
    /// This method returns the current animation frame with styling applied.
    /// It's the standard bubbletea-rs view function that produces the visual output.
    /// Matches Go's View method exactly.
    ///
    /// # Returns
    ///
    /// Returns the styled string representation of the current frame.
    /// Returns "(error)" if the frame index is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::spinner::{new, with_spinner, LINE};
    ///
    /// let spinner = new(&[with_spinner(LINE.clone())]);
    /// let output = spinner.view();
    /// assert_eq!(output, "|"); // First frame of LINE spinner
    /// ```
    pub fn view(&self) -> String {
        if self.frame >= self.spinner.frames.len() {
            return "(error)".to_string();
        }

        self.style.render(&self.spinner.frames[self.frame])
    }
}

impl BubbleTeaModel for Model {
    fn init() -> (Self, std::option::Option<Cmd>) {
        let model = Self::new();
        let cmd = model.tick();
        (model, std::option::Option::Some(cmd))
    }

    fn update(&mut self, msg: Msg) -> std::option::Option<Cmd> {
        self.update(msg)
    }

    fn view(&self) -> String {
        self.view()
    }
}

/// Creates a new spinner with the given configuration options.
///
/// This is the main constructor function that implements the options pattern
/// for creating customized spinners. It matches Go's New function exactly
/// for full API compatibility.
///
/// # Arguments
///
/// * `opts` - Slice of SpinnerOption values to configure the spinner
///
/// # Examples
///
/// ```rust
/// use bubbletea_widgets::spinner::{new, with_spinner, with_style, DOT};
/// use lipgloss_extras::prelude::*;
///
/// // Create with default settings
/// let basic_spinner = new(&[]);
///
/// // Create with custom animation and styling
/// let fancy_spinner = new(&[
///     with_spinner(DOT.clone()),
///     with_style(Style::new().foreground(Color::from("cyan")))
/// ]);
/// ```
pub fn new(opts: &[SpinnerOption]) -> Model {
    Model::new_with_options(opts)
}

/// NewModel returns a model with default values - matches Go's deprecated NewModel.
#[deprecated(since = "0.0.7", note = "use new instead")]
pub fn new_model(opts: &[SpinnerOption]) -> Model {
    new(opts)
}

/// Tick is the command used to advance the spinner one frame - matches Go's deprecated Tick function.
#[deprecated(since = "0.0.7", note = "use Model::tick_msg instead")]
pub fn tick() -> TickMsg {
    TickMsg {
        time: std::time::SystemTime::now(),
        id: 0,
        tag: 0,
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use crate::spinner::{
        dot, line, new, new_model, tick, with_spinner, with_style, DOT, ELLIPSIS, GLOBE, HAMBURGER,
        JUMP, LINE, METER, MINI_DOT, MONKEY, MOON, POINTS, PULSE,
    };

    #[test]
    fn test_spinner_constants() {
        // Test that all spinner constants exist and have correct frame counts
        assert_eq!(LINE.frames.len(), 4);
        assert_eq!(DOT.frames.len(), 8);
        assert_eq!(MINI_DOT.frames.len(), 10);
        assert_eq!(JUMP.frames.len(), 7);
        assert_eq!(PULSE.frames.len(), 4);
        assert_eq!(POINTS.frames.len(), 4);
        assert_eq!(GLOBE.frames.len(), 3);
        assert_eq!(MOON.frames.len(), 8);
        assert_eq!(MONKEY.frames.len(), 3);
        assert_eq!(METER.frames.len(), 7);
        assert_eq!(HAMBURGER.frames.len(), 4);
        assert_eq!(ELLIPSIS.frames.len(), 4);
    }

    #[test]
    fn test_spinner_frames_match_go() {
        // Test that spinner frames match Go implementation exactly
        assert_eq!(LINE.frames, vec!["|", "/", "-", "\\"]);
        assert_eq!(
            DOT.frames,
            vec!["⣾ ", "⣽ ", "⣻ ", "⢿ ", "⡿ ", "⣟ ", "⣯ ", "⣷ "]
        );
        assert_eq!(
            MINI_DOT.frames,
            vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
        );
        assert_eq!(JUMP.frames, vec!["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"]);
        assert_eq!(PULSE.frames, vec!["█", "▓", "▒", "░"]);
        assert_eq!(POINTS.frames, vec!["∙∙∙", "●∙∙", "∙●∙", "∙∙●"]);
        assert_eq!(GLOBE.frames, vec!["🌍", "🌎", "🌏"]);
        assert_eq!(
            MOON.frames,
            vec!["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"]
        );
        assert_eq!(MONKEY.frames, vec!["🙈", "🙉", "🙊"]);
        assert_eq!(
            METER.frames,
            vec!["▱▱▱", "▰▱▱", "▰▰▱", "▰▰▰", "▰▰▱", "▰▱▱", "▱▱▱"]
        );
        assert_eq!(HAMBURGER.frames, vec!["☱", "☲", "☴", "☲"]);
        assert_eq!(ELLIPSIS.frames, vec!["", ".", "..", "..."]);
    }

    #[test]
    fn test_new_with_no_options() {
        // Test Go's: New()
        let spinner = new(&[]);
        assert!(spinner.id() > 0); // Should have a unique ID
        assert_eq!(spinner.spinner.frames, LINE.frames); // Should default to Line
    }

    #[test]
    fn test_new_with_spinner_option() {
        // Test Go's: New(WithSpinner(Dot))
        let spinner = new(&[with_spinner(DOT.clone())]);
        assert_eq!(spinner.spinner.frames, DOT.frames);
    }

    #[test]
    fn test_new_with_style_option() {
        // Test Go's: New(WithStyle(style))
        let style = Style::new().foreground(lipgloss::Color::from("red"));
        let _spinner = new(&[with_style(style.clone())]);
        // Note: Style comparison is complex, so we just verify it was set
        // In a real test, you'd verify the style was applied correctly
    }

    #[test]
    fn test_new_with_multiple_options() {
        // Test Go's: New(WithSpinner(Jump), WithStyle(style))
        let style = Style::new().foreground(lipgloss::Color::from("blue"));
        let spinner = new(&[with_spinner(JUMP.clone()), with_style(style.clone())]);
        assert_eq!(spinner.spinner.frames, JUMP.frames);
    }

    #[test]
    fn test_model_id() {
        // Test Go's: model.ID()
        let spinner1 = new(&[]);
        let spinner2 = new(&[]);

        // Each spinner should have unique IDs
        assert_ne!(spinner1.id(), spinner2.id());
        assert!(spinner1.id() > 0);
        assert!(spinner2.id() > 0);
    }

    #[test]
    fn test_model_tick_msg() {
        // Test Go's: model.Tick()
        let spinner = new(&[]);
        let tick_msg = spinner.tick_msg();

        assert_eq!(tick_msg.id, spinner.id());
        // Time should be recent (within last second)
        let now = std::time::SystemTime::now();
        let elapsed = now.duration_since(tick_msg.time).unwrap();
        assert!(elapsed.as_secs() < 1);
    }

    #[test]
    fn test_global_tick_deprecated() {
        // Test Go's deprecated: Tick()
        let tick_msg = tick();
        assert_eq!(tick_msg.id, 0); // Global tick has ID 0
    }

    #[test]
    fn test_update_with_wrong_id() {
        // Test that spinner rejects messages with wrong ID
        let mut spinner = new(&[]);
        let wrong_tick = TickMsg {
            time: std::time::SystemTime::now(),
            id: spinner.id() + 999, // Wrong ID
            tag: 0,
        };

        let result = spinner.update(Box::new(wrong_tick));
        assert!(result.is_none()); // Should reject
    }

    #[test]
    fn test_update_with_correct_id() {
        // Test that spinner accepts messages with correct ID
        let mut spinner = new(&[]);
        let correct_tick = TickMsg {
            time: std::time::SystemTime::now(),
            id: spinner.id(),
            tag: 0,
        };

        let result = spinner.update(Box::new(correct_tick));
        assert!(result.is_some()); // Should accept and return new tick
    }

    #[test]
    fn test_view_renders_correctly() {
        // Test Go's: model.View()
        let mut spinner = new(&[with_spinner(LINE.clone())]);

        // Initial view should show first frame
        let view = spinner.view();
        assert_eq!(view, "|"); // First frame of Line spinner

        // After update, should show next frame
        let tick_msg = spinner.tick_msg();
        spinner.update(Box::new(tick_msg));
        let view = spinner.view();
        assert_eq!(view, "/"); // Second frame of Line spinner
    }

    #[test]
    fn test_frame_wrapping() {
        // Test that frames wrap around correctly
        let mut spinner = new(&[with_spinner(LINE.clone())]); // 4 frames

        // Advance through all frames
        for expected_frame in &["|", "/", "-", "\\", "|"] {
            // Should wrap back to first
            let view = spinner.view();
            assert_eq!(view, *expected_frame);

            if expected_frame != &"|" || view == "|" {
                // Don't tick after last assertion
                let tick_msg = spinner.tick_msg();
                spinner.update(Box::new(tick_msg));
            }
        }
    }

    #[test]
    fn test_deprecated_functions() {
        // Test that deprecated function aliases work
        #[allow(deprecated)]
        {
            let spinner_line = line();
            assert_eq!(spinner_line.frames, LINE.frames);

            let spinner_dot = dot();
            assert_eq!(spinner_dot.frames, DOT.frames);

            let model = new_model(&[]);
            assert!(model.id() > 0);
        }
    }

    #[test]
    fn test_builder_methods_still_work() {
        // Test that existing builder methods still work for backward compatibility
        let spinner = Model::new()
            .with_spinner(PULSE.clone())
            .with_style(Style::new());

        assert_eq!(spinner.spinner.frames, PULSE.frames);
    }
}
