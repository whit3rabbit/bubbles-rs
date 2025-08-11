//! Key bindings for list component navigation and interaction.
//!
//! This module defines the `ListKeyMap` which contains all the key bindings used by the list component
//! for navigation, filtering, and interaction. The key bindings follow common terminal UI conventions
//! and vim-style navigation patterns.
//!
//! ## Navigation Keys
//!
//! - **Cursor Movement**: `↑/k` (up), `↓/j` (down)
//! - **Page Navigation**: `→/l/pgdn/f/d` (next page), `←/h/pgup/b/u` (prev page)
//! - **Jump Navigation**: `g/home` (go to start), `G/end` (go to end)
//!
//! ## Filtering Keys
//!
//! - **Start Filter**: `/` (enter filtering mode)
//! - **Clear Filter**: `esc` (clear active filter)
//! - **Cancel Filter**: `esc` (cancel filter input)
//! - **Accept Filter**: `enter/tab/↑/↓` (apply filter and continue)
//!
//! ## Help and Quit Keys
//!
//! - **Help**: `?` (show/hide help)
//! - **Quit**: `q/esc` (normal quit)
//! - **Force Quit**: `ctrl+c` (immediate quit)
//!
//! ## Example
//!
//! ```rust
//! use bubbletea_widgets::list::ListKeyMap;
//! use bubbletea_widgets::key::KeyMap;
//!
//! let keymap = ListKeyMap::default();
//! let help = keymap.short_help(); // Get key bindings for help display
//! ```

use crate::key;
use crossterm::event::KeyCode;

/// Key bindings for list navigation, filtering, help, and exit actions.
#[derive(Debug, Clone)]
pub struct ListKeyMap {
    /// Move selection up one item.
    pub cursor_up: key::Binding,
    /// Move selection down one item.
    pub cursor_down: key::Binding,
    /// Go to the next page of items.
    pub next_page: key::Binding,
    /// Go to the previous page of items.
    pub prev_page: key::Binding,
    /// Jump to the first item.
    pub go_to_start: key::Binding,
    /// Jump to the last item.
    pub go_to_end: key::Binding,
    /// Enter filtering mode.
    pub filter: key::Binding,
    /// Clear the active filter.
    pub clear_filter: key::Binding,
    /// Cancel filtering mode.
    pub cancel_filter: key::Binding,
    /// Accept/apply the current filter input.
    pub accept_filter: key::Binding,
    /// Show the full help panel.
    pub show_full_help: key::Binding,
    /// Close the full help panel.
    pub close_full_help: key::Binding,
    /// Quit.
    pub quit: key::Binding,
    /// Force quit.
    pub force_quit: key::Binding,
}

impl Default for ListKeyMap {
    fn default() -> Self {
        Self {
            cursor_up: key::Binding::new(vec![KeyCode::Up, KeyCode::Char('k')])
                .with_help("↑/k", "up"),
            cursor_down: key::Binding::new(vec![KeyCode::Down, KeyCode::Char('j')])
                .with_help("↓/j", "down"),
            next_page: key::Binding::new(vec![
                KeyCode::Right,
                KeyCode::Char('l'),
                KeyCode::PageDown,
                KeyCode::Char('f'),
                KeyCode::Char('d'),
            ])
            .with_help("→/l/pgdn", "next page"),
            prev_page: key::Binding::new(vec![
                KeyCode::Left,
                KeyCode::Char('h'),
                KeyCode::PageUp,
                KeyCode::Char('b'),
                KeyCode::Char('u'),
            ])
            .with_help("←/h/pgup", "prev page"),
            go_to_start: key::Binding::new(vec![KeyCode::Home, KeyCode::Char('g')])
                .with_help("g/home", "go to start"),
            go_to_end: key::Binding::new(vec![KeyCode::End, KeyCode::Char('G')])
                .with_help("G/end", "go to end"),
            filter: key::Binding::new(vec![KeyCode::Char('/')]).with_help("/", "filter"),
            clear_filter: key::Binding::new(vec![KeyCode::Esc]).with_help("esc", "clear filter"),
            cancel_filter: key::Binding::new(vec![KeyCode::Esc]).with_help("esc", "cancel"),
            // Simplify accept_filter: Enter, Tab, Up/Down
            accept_filter: key::Binding::new(vec![
                KeyCode::Enter,
                KeyCode::Tab,
                KeyCode::Up,
                KeyCode::Down,
            ])
            .with_help("enter", "apply filter"),
            show_full_help: key::Binding::new(vec![KeyCode::Char('?')]).with_help("?", "more"),
            close_full_help: key::Binding::new(vec![KeyCode::Char('?')])
                .with_help("?", "close help"),
            quit: key::Binding::new(vec![KeyCode::Char('q'), KeyCode::Esc]).with_help("q", "quit"),
            // Use parse string for ctrl+c via key module convenience
            force_quit: key::new_binding(vec![key::with_keys_str(&["ctrl+c"])])
                .with_help("ctrl+c", "force quit"),
        }
    }
}

impl key::KeyMap for ListKeyMap {
    fn short_help(&self) -> Vec<&key::Binding> {
        vec![&self.cursor_up, &self.cursor_down, &self.filter, &self.quit]
    }

    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![
            // Column 1: Primary Navigation
            vec![
                &self.cursor_up,
                &self.cursor_down,
                &self.next_page,
                &self.prev_page,
                &self.go_to_start,
                &self.go_to_end,
            ],
            // Column 2: Filtering Actions
            vec![
                &self.filter,
                &self.clear_filter,
                &self.accept_filter,
                &self.cancel_filter,
            ],
            // Column 3: Help and Quit
            vec![&self.show_full_help, &self.quit],
        ]
    }
}
