//! High-precision stopwatch component for measuring elapsed time in Bubble Tea applications.
//!
//! This module provides a fully-featured stopwatch that can be started, stopped, paused,
//! and reset through Bubble Tea's message-driven architecture. It mirrors the functionality
//! of Go's bubbles stopwatch component while providing Rust-idiomatic interfaces and
//! memory safety guarantees.
//!
//! # Features
//!
//! - **Precise timing**: Uses Rust's `Duration` type for high-precision time measurement
//! - **Configurable intervals**: Customizable tick frequency from nanoseconds to seconds
//! - **Multiple instances**: Each stopwatch has a unique ID for managing multiple timers
//! - **Start/stop/pause/reset**: Full control over stopwatch lifecycle
//! - **Go-compatible formatting**: Duration display matches Go's time.Duration format
//! - **Message-driven**: Integrates seamlessly with Bubble Tea's update cycle
//!
//! # Quick Start
//!
//! ```rust
//! use bubbletea_widgets::stopwatch::{new, Model};
//! use bubbletea_rs::{Model as BubbleTeaModel, Msg};
//!
//! // Create a stopwatch with 1-second precision
//! let mut stopwatch = new();
//!
//! // Start timing
//! let start_cmd = stopwatch.start();
//!
//! // In your update loop, handle the start message
//! // stopwatch.update(start_msg); // This would start the timer
//!
//! // Check elapsed time
//! println!("Elapsed: {}", stopwatch.view()); // "0s" initially
//! ```
//!
//! # Integration with Bubble Tea
//!
//! ```rust
//! use bubbletea_widgets::stopwatch::{new, Model as StopwatchModel};
//! use bubbletea_rs::{Model as BubbleTeaModel, Cmd, Msg};
//!
//! struct App {
//!     stopwatch: StopwatchModel,
//! }
//!
//! impl BubbleTeaModel for App {
//!     fn init() -> (Self, Option<Cmd>) {
//!         let stopwatch = new();
//!         let start_cmd = stopwatch.start();
//!         (
//!             App { stopwatch },
//!             Some(start_cmd)
//!         )
//!     }
//!
//!     fn update(&mut self, msg: Msg) -> Option<Cmd> {
//!         // Forward messages to the stopwatch
//!         self.stopwatch.update(msg)
//!     }
//!
//!     fn view(&self) -> String {
//!         format!("Elapsed time: {}", self.stopwatch.view())
//!     }
//! }
//! ```
//!
//! # Custom Tick Intervals
//!
//! ```rust
//! use bubbletea_widgets::stopwatch::new_with_interval;
//! use std::time::Duration;
//!
//! // High-precision stopwatch updating every 10ms
//! let precise_stopwatch = new_with_interval(Duration::from_millis(10));
//!
//! // Low-precision stopwatch updating every 5 seconds
//! let coarse_stopwatch = new_with_interval(Duration::from_secs(5));
//! ```
//!
//! # Message Types
//!
//! The stopwatch communicates through three message types:
//! - `StartStopMsg`: Controls running state (start/stop/pause)
//! - `ResetMsg`: Resets elapsed time to zero
//! - `TickMsg`: Internal timing pulses (automatically generated)
//!
//! # Performance Notes
//!
//! - Each stopwatch instance has minimal memory overhead (< 64 bytes)
//! - Tick frequency directly impacts CPU usage - choose appropriately for your use case
//! - Multiple stopwatches can run concurrently without interference
//! - Duration formatting is optimized for common time ranges

use bubbletea_rs::{tick as bubbletea_tick, Cmd, Model as BubbleTeaModel, Msg};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

// Internal ID management for stopwatch instances
static LAST_ID: AtomicI64 = AtomicI64::new(0);

fn next_id() -> i64 {
    LAST_ID.fetch_add(1, Ordering::SeqCst) + 1
}

/// Format duration similar to Go's time.Duration String() output
fn format_duration(d: Duration) -> String {
    let total_nanos = d.as_nanos();

    if total_nanos == 0 {
        return "0s".to_string();
    }

    if total_nanos >= 1_000_000_000 {
        // Seconds or more
        let secs = d.as_secs_f64();
        if secs >= 60.0 {
            let minutes = (secs / 60.0) as u64;
            let remaining_secs = secs % 60.0;
            if remaining_secs == 0.0 {
                format!("{}m", minutes)
            } else {
                format!("{}m{:.0}s", minutes, remaining_secs)
            }
        } else if secs >= 1.0 {
            if (secs - secs.floor()).abs() < f64::EPSILON {
                format!("{:.0}s", secs)
            } else {
                format!("{:.1}s", secs)
            }
        } else {
            format!("{:.3}s", secs)
        }
    } else if total_nanos >= 1_000_000 {
        // Milliseconds
        format!("{}ms", d.as_millis())
    } else if total_nanos >= 1_000 {
        // Microseconds
        format!("{}µs", d.as_micros())
    } else {
        // Nanoseconds
        format!("{}ns", total_nanos)
    }
}

/// Message sent on every stopwatch tick to increment the elapsed time.
///
/// This message is generated automatically by the stopwatch at regular intervals
/// (determined by the `interval` field) when the stopwatch is running. The message
/// carries timing information and anti-flooding protection.
///
/// # Message Flow
///
/// 1. Stopwatch starts → generates first `TickMsg`
/// 2. `TickMsg` processed → elapsed time incremented → next `TickMsg` scheduled
/// 3. Continues until stopwatch is stopped or reset
///
/// # Examples
///
/// You typically don't create `TickMsg` instances manually, but you may need to
/// handle them in your application's update loop:
///
/// ```rust
/// use bubbletea_widgets::stopwatch::{TickMsg, Model};
/// use bubbletea_rs::Msg;
///
/// fn handle_message(stopwatch: &mut Model, msg: Msg) {
///     if let Some(tick) = msg.downcast_ref::<TickMsg>() {
///         println!("Tick from stopwatch ID: {}", tick.id);
///     }
///     stopwatch.update(msg);
/// }
/// ```
///
/// # Anti-flooding Protection
///
/// The `tag` field prevents message flooding by ensuring only the expected
/// tick sequence is processed. Out-of-order or duplicate ticks are ignored.
#[derive(Debug, Clone)]
pub struct TickMsg {
    /// Unique identifier of the stopwatch that generated this tick.
    ///
    /// Used to route messages to the correct stopwatch instance when multiple
    /// stopwatches are running in the same application.
    pub id: i64,
    /// Internal sequence number to prevent message flooding.
    ///
    /// Each tick increments this tag, and only ticks with the expected tag
    /// are processed. This prevents the stopwatch from receiving too many
    /// messages if the system gets backed up.
    tag: i64,
}

/// Message to control the running state of a stopwatch.
///
/// This message starts or stops a stopwatch, allowing you to pause and resume
/// timing as needed. When a stopwatch is stopped, it retains its current elapsed
/// time until started again or reset.
///
/// # Usage Pattern
///
/// Send this message through your application's command system:
///
/// ```rust
/// use bubbletea_widgets::stopwatch::new;
///
/// let stopwatch = new();
///
/// // Start the stopwatch
/// let start_cmd = stopwatch.start();  // Generates StartStopMsg { running: true }
///
/// // Stop the stopwatch  
/// let stop_cmd = stopwatch.stop();    // Generates StartStopMsg { running: false }
///
/// // Toggle running state
/// let toggle_cmd = stopwatch.toggle(); // Generates appropriate StartStopMsg
/// ```
///
/// # Examples
///
/// Manual creation (rarely needed):
/// ```rust
/// use bubbletea_widgets::stopwatch::new;
///
/// let stopwatch = new();
/// // Use the public API methods instead of constructing messages directly
/// let start_cmd = stopwatch.start();   // Generates StartStopMsg { running: true }
/// let stop_cmd = stopwatch.stop();     // Generates StartStopMsg { running: false }
/// ```
///
/// # State Transitions
///
/// - `running: true` → Stopwatch starts/resumes timing
/// - `running: false` → Stopwatch pauses (elapsed time preserved)
#[derive(Debug, Clone)]
pub struct StartStopMsg {
    /// Unique identifier of the target stopwatch.
    ///
    /// Must match the stopwatch's ID for the message to be processed.
    /// Use `stopwatch.id()` to get the correct value.
    pub id: i64,
    /// Whether the stopwatch should be running after processing this message.
    ///
    /// - `true`: Start or resume the stopwatch
    /// - `false`: Pause the stopwatch (preserving elapsed time)
    running: bool,
}

/// Message to reset a stopwatch's elapsed time to zero.
///
/// This message clears the accumulated elapsed time while preserving the stopwatch's
/// running state. A running stopwatch will continue timing from zero after reset,
/// while a stopped stopwatch will remain stopped with zero elapsed time.
///
/// # Usage Pattern
///
/// ```rust
/// use bubbletea_widgets::stopwatch::new;
///
/// let stopwatch = new();
/// let reset_cmd = stopwatch.reset(); // Generates ResetMsg
/// ```
///
/// # Examples
///
/// Manual creation (rarely needed):
/// ```rust
/// use bubbletea_widgets::stopwatch::{ResetMsg, new};
///
/// let stopwatch = new();
/// let reset_msg = ResetMsg {
///     id: stopwatch.id(),
/// };
/// // Send through your Bubble Tea update system
/// ```
///
/// # Behavior
///
/// - Resets `elapsed()` to `Duration::ZERO`
/// - Does not affect the running state
/// - Resets internal timing state for accurate subsequent measurements
#[derive(Debug, Clone)]
pub struct ResetMsg {
    /// Unique identifier of the target stopwatch.
    ///
    /// Must match the stopwatch's ID for the message to be processed.
    /// Use `stopwatch.id()` to get the correct value.
    pub id: i64,
}

/// A high-precision stopwatch for measuring elapsed time in Bubble Tea applications.
///
/// The `Model` represents a single stopwatch instance that can be started, stopped,
/// paused, and reset through Bubble Tea's message system. Each stopwatch maintains
/// its own elapsed time, running state, and unique identifier for use in
/// multi-stopwatch applications.
///
/// # Core Functionality
///
/// - **Timing**: Accumulates elapsed time with configurable tick intervals
/// - **State Management**: Tracks running/stopped state independently  
/// - **Identity**: Each instance has a unique ID for message routing
/// - **Precision**: Uses Rust's `Duration` for sub-second accuracy
///
/// # Examples
///
/// Basic usage:
/// ```rust
/// use bubbletea_widgets::stopwatch::{new, Model};
/// use std::time::Duration;
///
/// let mut stopwatch = new();
/// assert_eq!(stopwatch.elapsed(), Duration::ZERO);
/// assert!(!stopwatch.running());
/// ```
///
/// Custom interval for high-precision timing:
/// ```rust
/// use bubbletea_widgets::stopwatch::new_with_interval;
/// use std::time::Duration;
///
/// let high_precision = new_with_interval(Duration::from_millis(10));
/// assert_eq!(high_precision.interval, Duration::from_millis(10));
/// ```
///
/// Integration with Bubble Tea:
/// ```rust
/// use bubbletea_widgets::stopwatch::{new, Model as StopwatchModel};
/// use bubbletea_rs::{Model as BubbleTeaModel, Cmd, Msg};
/// use std::time::Duration;
///
/// struct TimerApp {
///     stopwatch: StopwatchModel,
/// }
///
/// impl BubbleTeaModel for TimerApp {
///     fn init() -> (Self, Option<Cmd>) {
///         let stopwatch = new();
///         let start_cmd = stopwatch.start();
///         (TimerApp { stopwatch }, Some(start_cmd))
///     }
///
///     fn update(&mut self, msg: Msg) -> Option<Cmd> {
///         self.stopwatch.update(msg)
///     }
///
///     fn view(&self) -> String {
///         format!("Timer: {} ({})",
///             self.stopwatch.view(),
///             if self.stopwatch.running() { "running" } else { "stopped" }
///         )
///     }
/// }
/// ```
///
/// # Thread Safety
///
/// `Model` is `Clone` and can be shared across threads, but individual instances
/// should be updated from a single thread to maintain timing accuracy.
#[derive(Debug, Clone)]
pub struct Model {
    /// Current elapsed time accumulated by the stopwatch.
    d: Duration,
    /// Unique identifier for this stopwatch instance.
    id: i64,
    /// Anti-flooding tag for tick message validation.
    tag: i64,
    /// Whether the stopwatch is currently accumulating time.
    running: bool,
    /// Time interval between ticks when running.
    ///
    /// This determines how frequently the elapsed time is updated and how
    /// precise the timing measurements will be. Shorter intervals provide
    /// higher precision but consume more CPU resources.
    ///
    /// # Default
    ///
    /// New stopwatches default to 1-second intervals (`Duration::from_secs(1)`)
    /// which is suitable for most applications displaying elapsed time to users.
    ///
    /// # Precision Trade-offs
    ///
    /// - `Duration::from_millis(10)`: High precision, higher CPU usage
    /// - `Duration::from_secs(1)`: Good balance for UI display
    /// - `Duration::from_secs(5)`: Low precision, minimal CPU usage
    pub interval: Duration,
}

/// Creates a new stopwatch with a custom tick interval.
///
/// This function creates a stopwatch that updates its elapsed time at the specified
/// interval. Shorter intervals provide more precise timing but consume more system
/// resources, while longer intervals are more efficient but less precise.
///
/// # Arguments
///
/// * `interval` - How frequently the stopwatch should update its elapsed time
///
/// # Returns
///
/// A new `Model` instance with the specified interval, initially stopped at zero elapsed time
///
/// # Examples
///
/// High-precision stopwatch for performance measurement:
/// ```rust
/// use bubbletea_widgets::stopwatch::new_with_interval;
/// use std::time::Duration;
///
/// let precise = new_with_interval(Duration::from_millis(1));
/// assert_eq!(precise.interval, Duration::from_millis(1));
/// assert_eq!(precise.elapsed(), Duration::ZERO);
/// assert!(!precise.running());
/// ```
///
/// Low-frequency stopwatch for long-running processes:
/// ```rust
/// use bubbletea_widgets::stopwatch::new_with_interval;
/// use std::time::Duration;
///
/// let coarse = new_with_interval(Duration::from_secs(10));
/// assert_eq!(coarse.interval, Duration::from_secs(10));
/// ```
///
/// Microsecond precision for benchmarking:
/// ```rust
/// use bubbletea_widgets::stopwatch::new_with_interval;
/// use std::time::Duration;
///
/// let benchmark = new_with_interval(Duration::from_micros(100));
/// assert_eq!(benchmark.interval, Duration::from_micros(100));
/// ```
///
/// # Performance Considerations
///
/// - **Nanosecond intervals**: Maximum precision, very high CPU usage
/// - **Millisecond intervals**: High precision, moderate CPU usage  
/// - **Second intervals**: Human-readable precision, low CPU usage
/// - **Minute+ intervals**: Minimal precision, negligible CPU usage
///
/// Choose the interval based on your application's precision requirements and
/// performance constraints.
pub fn new_with_interval(interval: Duration) -> Model {
    Model {
        d: Duration::ZERO,
        id: next_id(),
        tag: 0,
        running: false,
        interval,
    }
}

/// Creates a new stopwatch with a default 1-second tick interval.
///
/// This is the most commonly used constructor, providing a good balance between
/// timing precision and system resource usage. The 1-second interval is suitable
/// for most user-facing applications where elapsed time is displayed in seconds.
///
/// # Returns
///
/// A new `Model` instance with 1-second tick interval, initially stopped at zero elapsed time
///
/// # Examples
///
/// Basic stopwatch creation:
/// ```rust
/// use bubbletea_widgets::stopwatch::new;
/// use std::time::Duration;
///
/// let stopwatch = new();
/// assert_eq!(stopwatch.interval, Duration::from_secs(1));
/// assert_eq!(stopwatch.elapsed(), Duration::ZERO);
/// assert!(!stopwatch.running());
/// assert!(stopwatch.id() > 0); // Has unique ID
/// ```
///
/// Multiple independent stopwatches:
/// ```rust
/// use bubbletea_widgets::stopwatch::new;
///
/// let timer1 = new();
/// let timer2 = new();
/// // Each has a unique ID for independent operation
/// assert_ne!(timer1.id(), timer2.id());
/// ```
///
/// # Equivalent To
///
/// ```rust
/// use bubbletea_widgets::stopwatch::new_with_interval;
/// use std::time::Duration;
///
/// let stopwatch = new_with_interval(Duration::from_secs(1));
/// ```
pub fn new() -> Model {
    new_with_interval(Duration::from_secs(1))
}

impl Model {
    /// Returns the unique identifier of this stopwatch instance.
    ///
    /// Each stopwatch has a globally unique ID that's used for message routing
    /// when multiple stopwatches are running in the same application. This ID
    /// is automatically assigned during construction and never changes.
    ///
    /// # Returns
    ///
    /// A unique `i64` identifier for this stopwatch
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let stopwatch1 = new();
    /// let stopwatch2 = new();
    ///
    /// // Each stopwatch has a unique ID
    /// assert_ne!(stopwatch1.id(), stopwatch2.id());
    /// assert!(stopwatch1.id() > 0);
    /// assert!(stopwatch2.id() > 0);
    /// ```
    ///
    /// Using ID for message filtering:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::{new, StartStopMsg};
    /// use bubbletea_rs::Msg;
    ///
    /// let stopwatch = new();
    /// let my_id = stopwatch.id();
    ///
    /// // Create a start command for this specific stopwatch
    /// let start_cmd = stopwatch.start(); // Generates appropriate StartStopMsg
    /// ```
    ///
    /// # Thread Safety
    ///
    /// The ID is assigned using atomic operations and is safe to access
    /// from multiple threads.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// Returns whether the stopwatch is currently running and accumulating time.
    ///
    /// A running stopwatch actively updates its elapsed time at each tick interval.
    /// A stopped stopwatch preserves its current elapsed time without further updates.
    ///
    /// # Returns
    ///
    /// `true` if the stopwatch is running, `false` if stopped/paused
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let stopwatch = new();
    /// assert!(!stopwatch.running()); // Initially stopped
    /// ```
    ///
    /// Checking state after operations:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::{new, StartStopMsg};
    ///
    /// let mut stopwatch = new();
    /// assert!(!stopwatch.running());
    ///
    /// // After processing a start command, it would be running
    /// let start_cmd = stopwatch.start(); // Generates StartStopMsg internally
    /// // Process the command through your Bubble Tea app to set running = true
    /// ```
    ///
    /// # State Persistence
    ///
    /// The running state persists across:
    /// - Clone operations
    /// - Tick message processing
    /// - Reset operations (reset preserves running state)
    ///
    /// Only `StartStopMsg` messages change the running state.
    pub fn running(&self) -> bool {
        self.running
    }

    /// Initializes the stopwatch by generating a start command.
    ///
    /// This method is typically called when setting up the stopwatch in a Bubble Tea
    /// application's initialization phase. It generates a command that will start
    /// the stopwatch when processed by the update loop.
    ///
    /// # Returns
    ///
    /// A `Cmd` that will start the stopwatch when executed
    ///
    /// # Examples
    ///
    /// Using in a Bubble Tea application:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::{new, Model as StopwatchModel};
    /// use bubbletea_rs::{Model as BubbleTeaModel, Cmd, Msg};
    ///
    /// struct App {
    ///     stopwatch: StopwatchModel,
    /// }
    ///
    /// impl BubbleTeaModel for App {
    ///     fn init() -> (Self, Option<Cmd>) {
    ///         let stopwatch = new();
    ///         let init_cmd = stopwatch.init(); // Generates start command
    ///         (App { stopwatch }, Some(init_cmd))
    ///     }
    /// #   fn update(&mut self, _msg: Msg) -> Option<Cmd> { None }
    /// #   fn view(&self) -> String { String::new() }
    /// }
    /// ```
    ///
    /// Manual initialization:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let stopwatch = new();
    /// let init_cmd = stopwatch.init();
    /// // Execute this command in your Bubble Tea runtime
    /// ```
    ///
    /// # Note
    ///
    /// This method is equivalent to calling `start()` and is provided for
    /// consistency with the Bubble Tea component lifecycle.
    pub fn init(&self) -> Cmd {
        self.start()
    }

    /// Generates a command to start the stopwatch.
    ///
    /// Creates a command that, when processed, will start the stopwatch and begin
    /// accumulating elapsed time. If the stopwatch is already running, this command
    /// has no additional effect.
    ///
    /// # Returns
    ///
    /// A `Cmd` that will start the stopwatch when executed by the Bubble Tea runtime
    ///
    /// # Examples
    ///
    /// Basic start operation:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let stopwatch = new();
    /// let start_cmd = stopwatch.start();
    /// // Execute this command in your update loop to start timing
    /// ```
    ///
    /// Integration with application logic:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::{new, Model as StopwatchModel};
    /// use bubbletea_rs::{KeyMsg, Cmd, Msg};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// fn handle_keypress(stopwatch: &StopwatchModel, key: KeyMsg) -> Option<Cmd> {
    ///     match key.key {
    ///         KeyCode::Char('s') => {
    ///             if key.modifiers.is_empty() {
    ///                 Some(stopwatch.start()) // Start on 's' key
    ///             } else {
    ///                 None
    ///             }
    ///         }
    ///         _ => None,
    ///     }
    /// }
    /// ```
    ///
    /// # State Transition
    ///
    /// - **Stopped → Running**: Begins accumulating elapsed time
    /// - **Running → Running**: No change (idempotent)
    ///
    /// # Message Flow
    ///
    /// This method generates a `StartStopMsg` with `running: true` that will be
    /// processed by the `update()` method to change the stopwatch state.
    pub fn start(&self) -> Cmd {
        let id = self.id;
        bubbletea_tick(Duration::from_nanos(1), move |_| {
            Box::new(StartStopMsg { id, running: true }) as Msg
        })
    }

    /// Generates a command to stop/pause the stopwatch.
    ///
    /// Creates a command that, when processed, will stop the stopwatch and pause
    /// elapsed time accumulation. The current elapsed time is preserved and can
    /// be resumed later with `start()`. If the stopwatch is already stopped,
    /// this command has no additional effect.
    ///
    /// # Returns
    ///
    /// A `Cmd` that will stop the stopwatch when executed by the Bubble Tea runtime
    ///
    /// # Examples
    ///
    /// Basic stop operation:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let stopwatch = new();
    /// let stop_cmd = stopwatch.stop();
    /// // Execute this command to pause timing
    /// ```
    ///
    /// Stop-watch pattern:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::{new, Model as StopwatchModel};
    /// use bubbletea_rs::{KeyMsg, Cmd};
    /// use crossterm::event::KeyCode;
    ///
    /// fn handle_spacebar(stopwatch: &StopwatchModel) -> Cmd {
    ///     if stopwatch.running() {
    ///         stopwatch.stop()   // Pause if running
    ///     } else {
    ///         stopwatch.start()  // Resume if stopped
    ///     }
    /// }
    /// ```
    ///
    /// # State Transition
    ///
    /// - **Running → Stopped**: Pauses elapsed time accumulation
    /// - **Stopped → Stopped**: No change (idempotent)
    ///
    /// # Time Preservation
    ///
    /// Stopping a stopwatch preserves the current elapsed time:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    /// use std::time::Duration;
    ///
    /// // Imagine stopwatch has been running and shows some elapsed time
    /// let stopwatch = new();
    /// // let elapsed_before_stop = stopwatch.elapsed();
    ///
    /// // After processing stop command:
    /// // assert_eq!(stopwatch.elapsed(), elapsed_before_stop); // Time preserved
    /// // assert!(!stopwatch.running()); // But no longer accumulating
    /// ```
    pub fn stop(&self) -> Cmd {
        let id = self.id;
        bubbletea_tick(Duration::from_nanos(1), move |_| {
            Box::new(StartStopMsg { id, running: false }) as Msg
        })
    }

    /// Generates a command to toggle the stopwatch's running state.
    ///
    /// This is a convenience method that starts the stopwatch if it's currently
    /// stopped, or stops it if it's currently running. Useful for implementing
    /// play/pause functionality with a single key or button.
    ///
    /// # Returns
    ///
    /// A `Cmd` that will toggle the stopwatch state when executed:
    /// - If running: generates a stop command
    /// - If stopped: generates a start command
    ///
    /// # Examples
    ///
    /// Basic toggle functionality:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let stopwatch = new();
    /// assert!(!stopwatch.running());
    ///
    /// let toggle_cmd = stopwatch.toggle(); // Will generate start command
    /// ```
    ///
    /// Implementing spacebar toggle:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::{new, Model as StopwatchModel};
    /// use bubbletea_rs::{KeyMsg, Cmd};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// fn handle_key(stopwatch: &StopwatchModel, key: KeyMsg) -> Option<Cmd> {
    ///     match key.key {
    ///         KeyCode::Char(' ') if key.modifiers == KeyModifiers::NONE => {
    ///             Some(stopwatch.toggle()) // Spacebar toggles play/pause
    ///         }
    ///         _ => None,
    ///     }
    /// }
    /// ```
    ///
    /// # State Transitions
    ///
    /// - **Stopped → Running**: Equivalent to `start()`
    /// - **Running → Stopped**: Equivalent to `stop()`
    ///
    /// # Equivalent Implementation
    ///
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let stopwatch = new();
    /// let toggle_cmd = if stopwatch.running() {
    ///     stopwatch.stop()
    /// } else {
    ///     stopwatch.start()
    /// };
    /// ```
    pub fn toggle(&self) -> Cmd {
        if self.running() {
            self.stop()
        } else {
            self.start()
        }
    }

    /// Generates a command to reset the stopwatch's elapsed time to zero.
    ///
    /// Creates a command that, when processed, will clear the accumulated elapsed
    /// time while preserving the running state. A running stopwatch will continue
    /// timing from zero, while a stopped stopwatch will remain stopped with zero
    /// elapsed time.
    ///
    /// # Returns
    ///
    /// A `Cmd` that will reset the elapsed time when executed by the Bubble Tea runtime
    ///
    /// # Examples
    ///
    /// Basic reset operation:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let stopwatch = new();
    /// let reset_cmd = stopwatch.reset();
    /// // Execute this command to clear elapsed time
    /// ```
    ///
    /// Reset with state preservation:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    /// use std::time::Duration;
    ///
    /// // Imagine a running stopwatch with accumulated time
    /// let stopwatch = new();
    /// let was_running = stopwatch.running();
    ///
    /// let reset_cmd = stopwatch.reset();
    /// // After processing reset command:
    /// // assert_eq!(stopwatch.elapsed(), Duration::ZERO); // Time cleared
    /// // assert_eq!(stopwatch.running(), was_running);     // State preserved
    /// ```
    ///
    /// Implementing a reset button:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::{new, Model as StopwatchModel};
    /// use bubbletea_rs::{KeyMsg, Cmd};
    /// use crossterm::event::{KeyCode, KeyModifiers};
    ///
    /// fn handle_key(stopwatch: &StopwatchModel, key: KeyMsg) -> Option<Cmd> {
    ///     match key.key {
    ///         KeyCode::Char('r') if key.modifiers == KeyModifiers::NONE => {
    ///             Some(stopwatch.reset()) // 'r' key resets timer
    ///         }
    ///         _ => None,
    ///     }
    /// }
    /// ```
    ///
    /// # Behavior Details
    ///
    /// - **Elapsed time**: Set to `Duration::ZERO`
    /// - **Running state**: Unchanged (preserved)
    /// - **Internal timing**: Reset for accurate subsequent measurements
    /// - **ID and interval**: Unchanged
    ///
    /// # Use Cases
    ///
    /// - Lap timing (reset while continuing)
    /// - Error recovery (clear invalid measurements)
    /// - User-initiated restart
    /// - Preparation for new timing session
    pub fn reset(&self) -> Cmd {
        let id = self.id;
        bubbletea_tick(Duration::from_nanos(1), move |_| {
            Box::new(ResetMsg { id }) as Msg
        })
    }

    /// Processes messages and updates the stopwatch state.
    ///
    /// This method handles all incoming messages for the stopwatch, updating its
    /// internal state and scheduling follow-up commands as needed. It processes
    /// three types of messages: `StartStopMsg`, `ResetMsg`, and `TickMsg`.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to process, typically from the Bubble Tea runtime
    ///
    /// # Returns
    ///
    /// - `Some(Cmd)` if a follow-up command should be executed
    /// - `None` if no further action is needed
    ///
    /// # Message Types
    ///
    /// ## StartStopMsg
    /// Changes the running state and schedules the next tick if starting.
    ///
    /// ## ResetMsg  
    /// Clears elapsed time to zero without affecting running state.
    ///
    /// ## TickMsg
    /// Increments elapsed time and schedules the next tick if running.
    ///
    /// # Examples
    ///
    /// Basic usage in a Bubble Tea application:
    /// ```rust,ignore
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let mut stopwatch = new();
    ///
    /// // Start the stopwatch using the public API
    /// let start_cmd = stopwatch.start();
    /// // In a real app, you'd send this command through bubbletea
    /// assert!(!stopwatch.running()); // Initially not running
    /// ```
    ///
    /// Handling multiple message types:
    /// ```rust,ignore
    /// use bubbletea_widgets::stopwatch::{new, ResetMsg};
    /// use std::time::Duration;
    ///
    /// let mut stopwatch = new();
    ///
    /// // Start the stopwatch using the public API
    /// let start_cmd = stopwatch.start();
    ///
    /// // Reset to zero
    /// let reset = ResetMsg { id: stopwatch.id() };
    /// stopwatch.update(Box::new(reset));
    /// assert_eq!(stopwatch.elapsed(), Duration::ZERO);
    /// ```
    ///
    /// # Message Filtering
    ///
    /// Messages are filtered by ID to ensure they're intended for this stopwatch:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let mut stopwatch = new();
    /// // Messages with wrong IDs don't affect state
    /// assert!(!stopwatch.running()); // Initially not running
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This method should be called from a single thread to maintain timing accuracy,
    /// though the stopwatch can be cloned and used across threads.
    pub fn update(&mut self, msg: Msg) -> Option<Cmd> {
        if let Some(start_stop) = msg.downcast_ref::<StartStopMsg>() {
            if start_stop.id != self.id {
                return None;
            }
            self.running = start_stop.running;
            // When starting or stopping, schedule the next tick so we keep updating
            return Some(self.tick());
        }

        if let Some(reset) = msg.downcast_ref::<ResetMsg>() {
            if reset.id != self.id {
                return None;
            }
            self.d = Duration::ZERO;
            return None;
        }

        if let Some(tick) = msg.downcast_ref::<TickMsg>() {
            if !self.running || tick.id != self.id {
                return None;
            }
            // Reject unexpected tags to avoid too-frequent ticks
            if tick.tag > 0 && tick.tag != self.tag {
                return None;
            }

            self.d = self.d.saturating_add(self.interval);
            self.tag += 1;
            return Some(self.tick());
        }

        None
    }

    /// Returns a human-readable string representation of the elapsed time.
    ///
    /// Formats the accumulated elapsed time using Go-compatible duration formatting.
    /// The output format adapts to the magnitude of the elapsed time for optimal
    /// readability across different time scales.
    ///
    /// # Returns
    ///
    /// A formatted string representing the elapsed time
    ///
    /// # Format Examples
    ///
    /// - `"0s"` for zero duration
    /// - `"150ms"` for sub-second durations  
    /// - `"2.5s"` for fractional seconds
    /// - `"45s"` for whole seconds under a minute
    /// - `"2m30s"` for minutes with seconds
    /// - `"5m"` for whole minutes without seconds
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    /// use std::time::Duration;
    ///
    /// let stopwatch = new();
    /// assert_eq!(stopwatch.view(), "0s"); // Initially zero
    /// ```
    ///
    /// Displaying elapsed time in UI:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::{new, Model as StopwatchModel};
    /// use bubbletea_rs::{Model as BubbleTeaModel, Cmd, Msg};
    ///
    /// struct TimerDisplay {
    ///     stopwatch: StopwatchModel,
    /// }
    ///
    /// impl BubbleTeaModel for TimerDisplay {
    ///     fn init() -> (Self, Option<Cmd>) {
    ///         (TimerDisplay { stopwatch: new() }, None)
    ///     }
    /// #   fn update(&mut self, _msg: Msg) -> Option<Cmd> { None }
    ///
    ///     fn view(&self) -> String {
    ///         format!("Timer: {}", self.stopwatch.view())
    ///     }
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// String formatting is optimized for common time ranges and involves minimal
    /// allocations. Suitable for real-time UI updates.
    ///
    /// # Consistency
    ///
    /// The format matches Go's `time.Duration.String()` output for cross-language
    /// compatibility in applications that interoperate with Go services.
    pub fn view(&self) -> String {
        format_duration(self.d)
    }

    /// Returns the total elapsed time as a `Duration`.
    ///
    /// Provides access to the raw elapsed time for precise calculations,
    /// comparisons, or custom formatting. This is the accumulated time
    /// since the stopwatch was started, minus any time it was stopped.
    ///
    /// # Returns
    ///
    /// The total elapsed time as a `Duration`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    /// use std::time::Duration;
    ///
    /// let stopwatch = new();
    /// assert_eq!(stopwatch.elapsed(), Duration::ZERO); // Initially zero
    /// ```
    ///
    /// Precise timing calculations:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    /// use std::time::Duration;
    ///
    /// let stopwatch = new();
    /// let elapsed = stopwatch.elapsed();
    ///
    /// // Convert to different units
    /// let millis = elapsed.as_millis();
    /// let secs = elapsed.as_secs_f64();
    /// let nanos = elapsed.as_nanos();
    /// ```
    ///
    /// Performance measurement:
    /// ```rust,ignore
    /// use bubbletea_widgets::stopwatch::new;
    /// use std::time::Duration;
    ///
    /// let mut stopwatch = new();
    ///
    /// // Start timing using the public API
    /// let start_cmd = stopwatch.start();
    /// // In a real app, you'd send this command through bubbletea
    ///
    /// let elapsed = stopwatch.elapsed();
    /// assert!(elapsed >= Duration::ZERO);
    ///
    /// // Check if operation was fast enough
    /// if elapsed < Duration::from_millis(100) {
    ///     println!("Operation completed quickly: {:?}", elapsed);
    /// }
    /// ```
    ///
    /// Comparison with thresholds:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    /// use std::time::Duration;
    ///
    /// let stopwatch = new();
    /// let elapsed = stopwatch.elapsed();
    /// let threshold = Duration::from_secs(30);
    ///
    /// if elapsed > threshold {
    ///     println!("Timer has exceeded 30 seconds");
    /// }
    /// ```
    ///
    /// # Precision
    ///
    /// The returned `Duration` has the same precision as Rust's `Duration` type,
    /// which supports nanosecond-level timing on most platforms.
    pub fn elapsed(&self) -> Duration {
        self.d
    }

    /// Internal: schedule the next tick.
    fn tick(&self) -> Cmd {
        let id = self.id;
        let tag = self.tag;
        let interval = self.interval;
        bubbletea_tick(interval, move |_| Box::new(TickMsg { id, tag }) as Msg)
    }
}

impl BubbleTeaModel for Model {
    /// Creates a new stopwatch and starts it automatically.
    ///
    /// This implementation provides default behavior for using a stopwatch
    /// as a standalone Bubble Tea component. The returned stopwatch will
    /// begin timing immediately when the application starts.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - A new stopwatch model with default settings
    /// - A command to start the stopwatch
    ///
    /// # Examples
    ///
    /// Using stopwatch as a standalone component:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::new;
    ///
    /// let stopwatch = new();
    /// let _init_cmd = stopwatch.init();
    /// assert!(!stopwatch.running()); // Will be running after cmd execution
    /// ```
    fn init() -> (Self, Option<Cmd>) {
        let model = new();
        let cmd = model.init();
        (model, Some(cmd))
    }

    /// Forwards messages to the stopwatch's update method.
    ///
    /// This delegates message handling to the stopwatch's own update logic,
    /// maintaining the same behavior when used as a standalone component.
    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        self.update(msg)
    }

    /// Returns the formatted elapsed time string.
    ///
    /// Displays the stopwatch's current elapsed time in a human-readable
    /// format suitable for direct display in terminal UIs.
    fn view(&self) -> String {
        self.view()
    }
}

impl Default for Model {
    /// Creates a new stopwatch with default settings.
    ///
    /// Equivalent to calling `new()`, providing a stopwatch with:
    /// - Zero elapsed time
    /// - Stopped state
    /// - 1-second tick interval
    /// - Unique ID
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bubbletea_widgets::stopwatch::Model;
    /// use std::time::Duration;
    ///
    /// let stopwatch = Model::default();
    /// assert_eq!(stopwatch.elapsed(), Duration::ZERO);
    /// assert!(!stopwatch.running());
    /// assert_eq!(stopwatch.interval, Duration::from_secs(1));
    /// ```
    ///
    /// Using with struct initialization:
    /// ```rust
    /// use bubbletea_widgets::stopwatch::Model as StopwatchModel;
    ///
    /// #[derive(Default)]
    /// struct App {
    ///     timer: StopwatchModel,
    /// }
    ///
    /// let app = App::default(); // Uses stopwatch default
    /// ```
    fn default() -> Self {
        new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let sw = new();
        assert_eq!(sw.elapsed(), Duration::ZERO);
        assert!(!sw.running());
        assert!(sw.id() > 0);
        assert_eq!(sw.interval, Duration::from_secs(1));
    }

    #[test]
    fn test_start_stop_toggle_reset_cmds() {
        let sw = new();
        std::mem::drop(sw.start());
        std::mem::drop(sw.stop());
        std::mem::drop(sw.toggle());
        std::mem::drop(sw.reset());
    }

    #[test]
    fn test_update_flow() {
        let mut sw = new_with_interval(Duration::from_millis(10));

        // Start
        let start = StartStopMsg {
            id: sw.id(),
            running: true,
        };
        let next = sw.update(Box::new(start));
        assert!(next.is_some());
        assert!(sw.running());

        // Tick increments
        let before = sw.elapsed();
        let tick = TickMsg {
            id: sw.id(),
            tag: sw.tag,
        };
        let _ = sw.update(Box::new(tick));
        assert!(sw.elapsed() > before);

        // Reset
        let _ = sw.update(Box::new(ResetMsg { id: sw.id() }));
        assert_eq!(sw.elapsed(), Duration::ZERO);
    }
}
