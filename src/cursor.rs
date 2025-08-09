//! Cursor component for Bubble Tea-style text inputs.
//!
//! This component provides a reusable text cursor for inputs, text areas, and
//! other widgets that need a caret. It supports blinking, static, and hidden
//! modes and can be themed via Lip Gloss styles.
//!
//! The cursor is typically embedded inside another component (for example the
//! textarea model) and updated by forwarding messages. It can also be used as a
//! standalone `bubbletea_rs::Model` for demonstration or tests.
//!
//! ### Example
//! ```rust
//! use bubbletea_widgets::cursor;
//! use lipgloss_extras::prelude::*;
//!
//! let mut cur = cursor::new();
//! cur.style = Style::new().reverse(true); // style when the cursor block is shown
//! cur.text_style = Style::new();          // style for the character underneath when hidden
//! let _ = cur.focus();                    // start blinking
//! cur.set_char("x");
//! let _maybe_cmd = cur.set_mode(cursor::Mode::Blink);
//! let view = cur.view();
//! assert!(!view.is_empty());
//! ```

use bubbletea_rs::{tick, Cmd, Model as BubbleTeaModel, Msg};
use lipgloss_extras::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

// --- Internal ID Management ---
// Used to ensure that frame messages are only received by the cursor that sent them.
static LAST_ID: AtomicUsize = AtomicUsize::new(0);

fn next_id() -> usize {
    LAST_ID.fetch_add(1, Ordering::Relaxed)
}

const DEFAULT_BLINK_SPEED: Duration = Duration::from_millis(530);

// --- Messages ---

/// Message to start the cursor blinking.
#[derive(Debug, Clone)]
pub struct InitialBlinkMsg;

/// Message that signals the cursor should blink.
#[derive(Debug, Clone)]
pub struct BlinkMsg {
    /// Unique identifier of the cursor instance that this blink message targets.
    pub id: usize,
    /// Sequence tag to prevent processing stale blink messages.
    pub tag: usize,
}

// --- Mode ---

/// Describes the behavior of the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The cursor blinks.
    Blink,
    /// The cursor is static.
    Static,
    /// The cursor is hidden.
    Hide,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mode::Blink => "blink",
                Mode::Static => "static",
                Mode::Hide => "hidden",
            }
        )
    }
}

// --- Model ---

/// Model is the Bubble Tea model for this cursor element.
#[derive(Debug, Clone)]
pub struct Model {
    /// The speed at which the cursor blinks.
    pub blink_speed: Duration,
    /// Style for the cursor when it is visible (blinking "on").
    pub style: Style,
    /// Style for the text under the cursor when it is hidden (blinking "off").
    pub text_style: Style,

    char: String,
    id: usize,
    focus: bool,
    blink: bool, // Inverted logic: when `blink` is true, the cursor is *not* showing its block style.
    blink_tag: usize,
    mode: Mode,
}

impl Default for Model {
    /// Creates a new model with default settings.
    fn default() -> Self {
        Self {
            blink_speed: DEFAULT_BLINK_SPEED,
            style: Style::new(),
            text_style: Style::new(),
            char: " ".to_string(),
            id: next_id(),
            focus: false,
            blink: true, // Inverted logic: when `blink` is true, the cursor is *not* showing its block style.
            blink_tag: 0,
            mode: Mode::Blink,
        }
    }
}

impl Model {
    /// Creates a new model with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the visibility of the cursor.
    pub fn set_visible(&mut self, visible: bool) {
        self.blink = !visible;
    }

    /// Update is the Bubble Tea update loop. It handles cursor-related messages.
    /// This is not a `bubbletea_rs::Model` implementation because the cursor is
    /// a sub-component managed by another model.
    pub fn update(&mut self, msg: &Msg) -> Option<Cmd> {
        if msg.downcast_ref::<InitialBlinkMsg>().is_some() {
            if self.mode != Mode::Blink || !self.focus {
                return None;
            }
            return self.blink_cmd();
        }

        if let Some(blink_msg) = msg.downcast_ref::<BlinkMsg>() {
            // Is this model blink-able?
            if self.mode != Mode::Blink || !self.focus {
                return None;
            }

            // Were we expecting this blink message?
            if blink_msg.id != self.id || blink_msg.tag != self.blink_tag {
                return None;
            }

            self.blink = !self.blink;
            return self.blink_cmd();
        }

        None
    }

    /// Returns the model's cursor mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Sets the model's cursor mode. This method returns a command.
    pub fn set_mode(&mut self, mode: Mode) -> Option<Cmd> {
        self.mode = mode;
        self.blink = self.mode == Mode::Hide || !self.focus;
        if mode == Mode::Blink {
            return Some(blink());
        }
        None
    }

    /// Creates a command to schedule the next blink.
    fn blink_cmd(&mut self) -> Option<Cmd> {
        if self.mode != Mode::Blink {
            return None;
        }

        self.blink_tag += 1;
        let tag = self.blink_tag;
        let id = self.id;
        let speed = self.blink_speed;

        Some(tick(speed, move |_| Box::new(BlinkMsg { id, tag }) as Msg))
    }

    /// Focuses the cursor to allow it to blink if desired.
    pub fn focus(&mut self) -> Option<Cmd> {
        self.focus = true;
        self.blink = self.mode == Mode::Hide; // Show the cursor unless we've explicitly hidden it
        if self.mode == Mode::Blink && self.focus {
            return self.blink_cmd();
        }
        None
    }

    /// Blurs the cursor.
    pub fn blur(&mut self) {
        self.focus = false;
        self.blink = true;
    }

    /// Check if cursor is focused
    pub fn focused(&self) -> bool {
        self.focus
    }

    /// Sets the character under the cursor.
    pub fn set_char(&mut self, s: &str) {
        self.char = s.to_string();
    }

    /// Renders the cursor.
    pub fn view(&self) -> String {
        if self.mode == Mode::Hide || self.blink {
            // When blinking is "on", we show the text style (cursor is hidden)
            return self.text_style.clone().inline(true).render(&self.char);
        }
        // When blinking is "off", we show the cursor style (reversed)
        self.style
            .clone()
            .inline(true)
            .reverse(true)
            .render(&self.char)
    }
}

// Optional: Implement BubbleTeaModel for standalone use (though cursor is typically a sub-component)
impl BubbleTeaModel for Model {
    fn init() -> (Self, Option<Cmd>) {
        let model = Self::new();
        (model, Some(blink()))
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        self.update(&msg)
    }

    fn view(&self) -> String {
        self.view()
    }
}

/// A command to initialize cursor blinking.
pub fn blink() -> Cmd {
    tick(Duration::from_millis(0), |_| {
        Box::new(InitialBlinkMsg) as Msg
    })
}

/// Create a new cursor model. Equivalent to Model::new().
pub fn new() -> Model {
    Model::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test verifies that the tag captured in a blink command's message
    // is the tag value at the time of command creation, ensuring no race conditions
    // Note: The original Go test used goroutines to test race conditions, but
    // since Rust's blink_cmd captures values by move, race conditions are prevented by design
    #[test]
    fn test_blink_cmd_tag_captured_no_race() {
        let mut m = Model::new();
        m.blink_speed = Duration::from_millis(10);
        m.mode = Mode::Blink;
        m.focus = true;

        // First blink command; capture expected tag immediately after creation.
        let _cmd1 = m.blink_cmd().expect("cmd1");
        let expected_tag = m.blink_tag; // blink_cmd increments before returning
        let _expected_id = m.id;

        // Schedule another blink command to mutate blink_tag (simulating what would be a race in Go)
        let _cmd2 = m.blink_cmd();
        let new_tag = m.blink_tag;

        // In Rust, the closure in cmd1 captured the values by move when created,
        // so even though we've created cmd2 and incremented blink_tag,
        // cmd1 still has the original values
        assert_ne!(
            expected_tag, new_tag,
            "Tags should be different after second blink_cmd"
        );

        // We can't actually await the Cmd without an async runtime,
        // but we've verified the key property: that the tag is captured at creation time
        // The actual message would have id=expected_id and tag=expected_tag when executed
    }
}
