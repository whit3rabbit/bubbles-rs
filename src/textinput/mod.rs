//! Text input component for Bubble Tea applications.
//!
//! Package textinput provides a text input component for Bubble Tea applications.
//! It closely matches the Go bubbles textinput component API for 1-1 compatibility.
//!
//! # Basic Usage
//!
//! ```rust
//! use bubbles_rs::textinput::{new, Model};
//! use bubbletea_rs::{Model as BubbleTeaModel, Msg, Cmd};
//!
//! // Create a text input with default settings
//! let mut input = new();
//! input.focus();
//!
//! // Set placeholder, width, and other options
//! input.set_placeholder("Enter your name...");
//! input.set_width(30);
//! ```
//!
//! # Echo Modes
//!
//! ```rust
//! use bubbles_rs::textinput::{new, EchoMode};
//!
//! let mut input = new();
//! input.set_echo_mode(EchoMode::EchoPassword); // Hide text with asterisks
//! ```
//!
//! # Key Bindings
//!
//! The component uses the key system for handling keyboard input with customizable bindings.
//!
//! # Testing
//!
//! This module includes comprehensive tests that match the Go implementation exactly.
//! Run tests with: `cargo test textinput` to verify 1-1 compatibility.

pub mod keymap;
pub mod methods;
pub mod model;
pub mod movement;
pub mod suggestions;
pub mod types;
pub mod view;

#[cfg(test)]
mod tests;

// Re-export main types and functions for public API
pub use keymap::{default_key_map, KeyMap};
pub use model::{blink, new, new_model, paste, Model};
pub use types::{EchoMode, PasteErrMsg, PasteMsg, ValidateFunc};
