# `bubbletea-widgets` API Documentation

Welcome to `bubbletea-widgets`, a collection of reusable, production-ready TUI components for building terminal applications with [bubbletea-rs](https://crates.io/crates/bubbletea-rs). This library is a Rust port of the popular Go library [bubbles](https://github.com/charmbracelet/bubbles).

Each component is designed to be self-contained and easy to integrate into your `bubbletea-rs` applications.

## Table of Contents
- [Installation](#installation)
- [Core Concepts](#core-concepts)
- [Components](#components)
  - [Key](#key)
  - [Spinner](#spinner)
  - [Progress](#progress)
  - [Timer](#timer)
  - [Stopwatch](#stopwatch)
  - [TextInput](#textinput)
  - [TextArea](#textarea)
  - [Paginator](#paginator)
  - [Viewport](#viewport)
  - [Help](#help)
  - [List](#list)
  - [Table](#table)
  - [File Picker](#file-picker)
  - [Cursor](#cursor)

## Installation

Add `bubble-rs` to your `Cargo.toml` dependencies. You will also need `bubbletea-rs` and `lipgloss` for a complete TUI application.

```toml
[dependencies]
bubble-rs = "0.0.6"
bubbletea-rs = "0.0.6"
lipgloss = "0.0.6"
```

## Core Concepts

Many components in `bubble-rs` implement the `Component` trait, which provides a standard interface for focus management.

```rust
pub trait Component {
    fn focus(&mut self) -> Option<Cmd>;
    fn blur(&mut self);
    fn focused(&self) -> bool;
}
```

This allows you to easily manage which part of your UI is currently active and receiving input.

---

## Components

### Key

The `key` module provides a robust way to manage keybindings. It allows you to define semantic actions (like "move up") and associate them with multiple physical key presses (e.g., `up` arrow and `k`). This is essential for building accessible applications and for generating help views.

**Public API:**
-   **`struct Binding`**: Represents a keybinding with associated keys, help text (`key` and `desc`), and an enabled/disabled state.
-   **`struct KeyPress`**: A type-safe representation of a key press, including `KeyCode` and `KeyModifiers`.
-   **`trait KeyMap`**: An interface for components to expose their keybindings for the `help` component.
-   **`fn new_binding(opts: Vec<BindingOpt>) -> Binding`**: Creates a new binding using a builder-like pattern.
-   **`fn matches(key_msg: &KeyMsg, bindings: &[&Binding]) -> bool`**: Checks if a `KeyMsg` matches any of the given bindings.

**Usage Example:**
```rust
use bubbletea_widgets::key::{self, Binding};
use bubbletea_rs::{KeyMsg, Model as BubbleTeaModel};
use crossterm::event::{KeyCode, KeyModifiers};

struct MyKeyMap {
    up: Binding,
    down: Binding,
    quit: Binding,
}

impl Default for MyKeyMap {
    fn default() -> Self {
        Self {
            up: Binding::new(vec![KeyCode::Up, KeyCode::Char('k')]).with_help("↑/k", "move up"),
            down: Binding::new(vec![KeyCode::Down, KeyCode::Char('j')]).with_help("↓/j", "move down"),
            quit: Binding::new(vec![KeyCode::Char('q'), (KeyCode::Char('c'), KeyModifiers::CONTROL).into()]).with_help("q/ctrl+c", "quit"),
        }
    }
}

struct App {
    keymap: MyKeyMap,
}

impl BubbleTeaModel for App {
    // ... init ...
    fn update(&mut self, msg: bubbletea_rs::Msg) -> Option<bubbletea_rs::Cmd> {
        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            if self.keymap.up.matches(key_msg) {
                // Handle "up" action
            } else if self.keymap.quit.matches(key_msg) {
                return Some(bubbletea_rs::quit());
            }
        }
        None
    }
    // ... view ...
}
```

---

### Spinner

A spinner indicates that an operation is in progress. It's highly customizable, with several built-in styles.

**Public API:**
-   **`struct Model`**: The spinner's state.
-   **`fn new(opts: &[SpinnerOption]) -> Model`**: Creates a new spinner.
-   **`enum SpinnerOption`**: Configuration options for the spinner (`with_spinner`, `with_style`).
-   **`struct Spinner`**: Defines the frames and speed of an animation.
-   **Constants**: Predefined spinners like `LINE`, `DOT`, `GLOBE`, `MONKEY`, etc.
-   **`Model::update(&mut self, msg: Msg)`**: Advances the spinner animation.
-   **`Model::view(&self)`**: Renders the current spinner frame.
-   **`Model::tick_msg(&self) -> TickMsg`**: Generates the message to start and continue the animation.

**Usage Example:**
```rust
use bubbletea_widgets::spinner::{self, with_spinner, with_style, TickMsg};
use bubbletea_rs::{Cmd, Model as BubbleTeaModel, Msg};
use lipgloss::{Color, Style};

struct App {
    spinner: spinner::Model,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<Cmd>) {
        let s = spinner::new(&[
            with_spinner(spinner::DOT.clone()),
            with_style(Style::new().foreground(Color::from("205"))),
        ]);
        let cmd = s.tick_msg().into(); // Start the spinner
        (Self { spinner: s }, Some(cmd))
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        // Pass the message to the spinner's update function
        self.spinner.update(msg)
    }

    fn view(&self) -> String {
        format!("{} Loading...", self.spinner.view())
    }
}
```

---

### Progress

A progress bar to visualize the completion of a task. It can be a solid color or a gradient and supports smooth animation.

**Public API:**
-   **`struct Model`**: The progress bar's state.
-   **`fn new(opts: &[ProgressOption]) -> Model`**: Creates a new progress bar.
-   **`enum ProgressOption`**: Configuration options like `with_width`, `with_gradient`, `with_solid_fill`, `without_percentage`.
-   **`Model::set_percent(&mut self, p: f64) -> Cmd`**: Sets the progress to a specific value and returns a command to start the animation.
-   **`Model::view(&self)`**: Renders the progress bar based on its *animated* state.
-   **`Model::view_as(&self, percent: f64)`**: Renders a static view of the progress bar at a given percentage.

**Usage Example:**
```rust
use bubbletea_widgets::progress::{self, with_width, FrameMsg};
use bubbletea_rs::{Cmd, Model as BubbleTeaModel, Msg};
use std::time::Duration;

struct App {
    progress: progress::Model,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<Cmd>) {
        let mut p = progress::new(&[with_width(40)]);
        // Set initial progress and start animation
        let cmd = p.set_percent(0.25);
        (Self { progress: p }, Some(cmd))
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        // The progress model handles its own animation via FrameMsg
        self.progress.update(msg)
    }

    fn view(&self) -> String {
        format!("Downloading...\n{}", self.progress.view())
    }
}
```

---

### Timer

A component for counting down from a specified duration.

**Public API:**
-   **`struct Model`**: The timer's state.
-   **`fn new(timeout: Duration) -> Model`**: Creates a new timer with a 1-second interval.
-   **`fn new_with_interval(timeout: Duration, interval: Duration) -> Model`**: Creates a timer with a custom interval.
-   **`Model::update(&mut self, msg: Msg)`**: Updates the timer's countdown.
-   **`Model::view(&self)`**: Renders the remaining time.
-   **Messages**: `TickMsg` (sent on each interval), `TimeoutMsg` (sent when the timer finishes), `StartStopMsg` (to control the timer).

**Usage Example:**
```rust
use bubbletea_widgets::timer::{self, TimeoutMsg};
use bubbletea_rs::{Cmd, Model as BubbleTeaModel, Msg};
use std::time::Duration;

struct App {
    timer: timer::Model,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<Cmd>) {
        let t = timer::new(Duration::from_secs(5));
        let cmd = t.init(); // Start the timer
        (Self { timer: t }, Some(cmd))
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        if let Some(timeout_msg) = msg.downcast_ref::<TimeoutMsg>() {
            if timeout_msg.id == self.timer.id() {
                // Timer finished!
                return Some(bubbletea_rs::quit());
            }
        }
        self.timer.update(msg)
    }

    fn view(&self) -> String {
        format!("Time left: {}", self.timer.view())
    }
}
```

---

### Stopwatch

A component for counting up from zero.

**Public API:**
-   **`struct Model`**: The stopwatch's state.
-   **`fn new() -> Model`**: Creates a new stopwatch with a 1-second interval.
-   **`fn new_with_interval(interval: Duration) -> Model`**: Creates a stopwatch with a custom interval.
-   **`Model::start(&self) -> Cmd`**: Returns a command to start or resume the stopwatch.
-   **`Model::stop(&self) -> Cmd`**: Returns a command to pause the stopwatch.
-   **`Model::reset(&self) -> Cmd`**: Returns a command to reset the stopwatch to zero.
-   **`Model::update(&mut self, msg: Msg)`**: Updates the stopwatch's elapsed time.
-   **`Model::view(&self)`**: Renders the elapsed time.

**Usage Example:**
```rust
use bubbletea_widgets::stopwatch;
use bubbletea_rs::{Cmd, Model as BubbleTeaModel, Msg};

struct App {
    stopwatch: stopwatch::Model,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<Cmd>) {
        let sw = stopwatch::new();
        let cmd = sw.start(); // Start the stopwatch
        (Self { stopwatch: sw }, Some(cmd))
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        self.stopwatch.update(msg)
    }

    fn view(&self) -> String {
        format!("Elapsed: {}", self.stopwatch.view())
    }
}
```

---

### TextInput

A single-line text input field, similar to an HTML `<input type="text">`.

**Public API:**
-   **`struct Model`**: The text input's state.
-   **`fn new() -> Model`**: Creates a new text input.
-   **`Model::focus(&mut self) -> Cmd`**: Focuses the input and returns a blink command.
-   **`Model::blur(&mut self)`**: Removes focus.
-   **`Model::set_value(&mut self, s: &str)`**: Sets the input's content.
-   **`Model::value(&self) -> String`**: Gets the input's content.
-   **`Model::set_placeholder(&mut self, p: &str)`**: Sets the placeholder text.
-   **`Model::set_echo_mode(&mut self, mode: EchoMode)`**: Changes the echo mode (e.g., `EchoPassword`).
-   **`Model::update(&mut self, msg: Msg)`**: Handles user input.
-   **`Model::view(&self)`**: Renders the text input.

**Usage Example:**
```rust
use bubbletea_widgets::textinput;
use bubbletea_rs::{Cmd, KeyMsg, Model as BubbleTeaModel, Msg};
use crossterm::event::KeyCode;

struct App {
    text_input: textinput::Model,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<Cmd>) {
        let mut ti = textinput::new();
        ti.set_placeholder("Enter your name...");
        let cmd = ti.focus();
        (Self { text_input: ti }, Some(cmd))
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            if key_msg.key == KeyCode::Enter {
                return Some(bubbletea_rs::quit());
            }
        }
        self.text_input.update(msg)
    }

    fn view(&self) -> String {
        format!("What's your name?\n{}\n\n(esc to quit)", self.text_input.view())
    }
}
```

---

### TextArea

A multi-line text input field, similar to an HTML `<textarea>`. It supports soft-wrapping, scrolling, and line numbers.

**Public API:**
-   **`struct Model`**: The text area's state.
-   **`fn new() -> Model`**: Creates a new text area.
-   **`Component` trait implementation**: `focus()`, `blur()`, `focused()`.
-   **`Model::set_value(&mut self, s: &str)`**: Sets the content.
-   **`Model::value(&self) -> String`**: Gets the content.
-   **`Model::set_placeholder(&mut self, p: &str)`**: Sets placeholder text.
-   **`Model::set_width(&mut self, w: usize)` / `set_height(&mut self, h: usize)`**: Sets dimensions.
-   **`Model::update(&mut self, msg: Option<Msg>)`**: Handles user input and events.
-   **`Model::view(&self)`**: Renders the text area.
-   **Fields**: `show_line_numbers: bool`, `key_map: TextareaKeyMap`, styling structs.

**Usage Example:**
```rust
use bubbletea_widgets::textarea;
use bubbletea_rs::{Cmd, Component, Model as BubbleTeaModel, Msg};

struct App {
    textarea: textarea::Model,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<Cmd>) {
        let mut ta = textarea::new();
        ta.set_placeholder("Write a short story...");
        ta.set_width(50);
        ta.set_height(5);
        let cmd = ta.focus();
        (Self { textarea: ta }, cmd)
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        self.textarea.update(Some(msg))
    }

    fn view(&self) -> String {
        format!("Tell me a story:\n{}\n\n(ctrl+c to quit)", self.textarea.view())
    }
}
```

---

### Paginator

A component for handling pagination logic and rendering pagination UI.

**Public API:**
-   **`struct Model`**: The paginator's state.
-   **`fn new() -> Model`**: Creates a new paginator.
-   **`Model::set_total_items(&mut self, items: usize)`**: Calculates total pages based on item count.
-   **`Model::set_per_page(&mut self, per_page: usize)`**: Sets how many items are on a page.
-   **`Model::next_page(&mut self)` / `prev_page(&mut self)`**: Navigates pages.
-   **`Model::update(&mut self, msg: &Msg)`**: Handles key presses for navigation.
-   **`Model::view(&self)`**: Renders the paginator UI (e.g., `1/10` or `● ○ ○`).
-   **Fields**: `page`, `per_page`, `total_pages`, `paginator_type: Type`.

**Usage Example:**
```rust
use bubbletea_widgets::paginator;
use bubbletea_rs::{KeyMsg, Model as BubbleTeaModel, Msg};

struct App {
    paginator: paginator::Model,
    items: Vec<String>,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<bubbletea_rs::Cmd>) {
        let items: Vec<String> = (1..=100).map(|i| format!("Item {}", i)).collect();
        let mut p = paginator::new();
        p.set_per_page(10);
        p.set_total_items(items.len());
        (Self { paginator: p, items }, None)
    }

    fn update(&mut self, msg: Msg) -> Option<bubbletea_rs::Cmd> {
        self.paginator.update(&msg);
        None
    }

    fn view(&self) -> String {
        let (start, end) = self.paginator.get_slice_bounds(self.items.len());
        let current_page_items = &self.items[start..end];

        format!(
            "Items on this page:\n{}\n\n{}",
            current_page_items.join("\n"),
            self.paginator.view()
        )
    }
}
```

---

### Viewport

A component for viewing and vertically scrolling large blocks of content.

**Public API:**
-   **`struct Model`**: The viewport's state.
-   **`fn new(width: usize, height: usize) -> Model`**: Creates a new viewport.
-   **`Model::set_content(&mut self, content: &str)`**: Sets the content to be displayed.
-   **`Model::scroll_up(&mut self, n: usize)` / `scroll_down(&mut self, n: usize)`**: Scrolls content.
-   **`Model::goto_top(&mut self)` / `goto_bottom(&mut self)`**: Jumps to the start or end.
-   **`Model::at_top(&self)` / `at_bottom(&self)`**: Checks if the viewport is at an edge.
-   **`Model::update(&mut self, msg: Msg)`**: Handles key presses for scrolling.
-   **`Model::view(&self)`**: Renders the visible portion of the content.

**Usage Example:**
```rust
use bubbletea_widgets::viewport;
use bubbletea_rs::{Model as BubbleTeaModel, Msg};

struct App {
    viewport: viewport::Model,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<bubbletea_rs::Cmd>) {
        let mut vp = viewport::new(80, 20);
        vp.set_content("A very long string with many\nlines of text...\n...that need to be scrolled.");
        (Self { viewport: vp }, None)
    }

    fn update(&mut self, msg: Msg) -> Option<bubbletea_rs::Cmd> {
        self.viewport.update(msg)
    }

    fn view(&self) -> String {
        self.viewport.view()
    }
}
```

---

### Help

A mini help view that automatically generates itself from a `KeyMap`.

**Public API:**
-   **`struct Model`**: The help view's state.
-   **`fn new() -> Model`**: Creates a new help model.
-   **`Model::view<K: KeyMap>(&self, keymap: &K)`**: Renders the help view based on the provided key map.
-   **Fields**: `show_all: bool` (toggles between short and full help), `width: usize`.
-   **`trait KeyMap`**: Must be implemented on your `KeyMap` struct to provide keys to the help view.

**Usage Example:**
```rust
use bubbletea_widgets::{help, key};
use bubbletea_rs::{KeyMsg, Model as BubbleTeaModel, Msg};
use crossterm::event::KeyCode;

// 1. Define your KeyMap
struct AppKeyMap {
    up: key::Binding,
    down: key::Binding,
    help: key::Binding,
}

// 2. Implement the help::KeyMap trait
impl help::KeyMap for AppKeyMap {
    fn short_help(&self) -> Vec<&key::Binding> {
        vec![&self.up, &self.down, &self.help]
    }
    fn full_help(&self) -> Vec<Vec<&key::Binding>> {
        vec![vec![&self.up, &self.down, &self.help]]
    }
}

struct App {
    keymap: AppKeyMap,
    help: help::Model,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<bubbletea_rs::Cmd>) {
        (
            Self {
                keymap: AppKeyMap {
                    up: key::Binding::new(vec![KeyCode::Up]).with_help("↑", "up"),
                    down: key::Binding::new(vec![KeyCode::Down]).with_help("↓", "down"),
                    help: key::Binding::new(vec![KeyCode::Char('?')]).with_help("?", "toggle help"),
                },
                help: help::new(),
            },
            None,
        )
    }

    fn update(&mut self, msg: Msg) -> Option<bubbletea_rs::Cmd> {
        if let Some(key_msg) = msg.downcast_ref::<KeyMsg>() {
            if self.keymap.help.matches(key_msg) {
                self.help.show_all = !self.help.show_all;
            }
        }
        None
    }

    fn view(&self) -> String {
        format!("Content...\n\n{}", self.help.view(&self.keymap))
    }
}
```

---

### List

A feature-rich component for browsing a set of items, with filtering, pagination, and status messages.

**Public API:**
-   **`struct Model<I: Item>`**: The list's state.
-   **`trait Item`**: A trait your list items must implement (`filter_value()`).
-   **`trait ItemDelegate`**: Defines how items are rendered and updated.
-   **`fn new(items: Vec<I>, delegate: impl ItemDelegate, width, height)`**: Creates a new list.
-   **`Model::update(&mut self, msg: Msg)`**: Handles all list interactions.
-   **`Model::view(&self)`**: Renders the entire list component.
-   **`Model::selected_item(&self) -> Option<&I>`**: Gets the currently selected item.
-   **Default Implementations**: `DefaultItem`, `DefaultDelegate`, and `DefaultItemStyles` are provided for common use cases.

**Usage Example:**
```rust
use bubbletea_widgets::list::{self, DefaultItem};
use bubbletea_rs::{Model as BubbleTeaModel, Msg};

struct App {
    list: list::Model<DefaultItem>,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<bubbletea_rs::Cmd>) {
        let items = vec![
            DefaultItem::new("Turtles", "They are slow"),
            DefaultItem::new("Snails", "They are slimy"),
            DefaultItem::new("Cats", "They are cute"),
        ];
        let delegate = list::DefaultDelegate::new();
        let list_model = list::Model::new(items, delegate, 40, 10);
        (Self { list: list_model }, None)
    }

    fn update(&mut self, msg: Msg) -> Option<bubbletea_rs::Cmd> {
        self.list.update(msg)
    }

    fn view(&self) -> String {
        self.list.view()
    }
}
```

---

### Table

A component for displaying and navigating tabular data.

**Public API:**
-   **`struct Model`**: The table's state.
-   **`fn new(columns: Vec<Column>) -> Model`**: Creates a new table with defined columns.
-   **`struct Column { title: String, width: i32 }`**: Defines a table column.
-   **`struct Row { cells: Vec<String> }`**: Defines a table row.
-   **`Model::with_rows(mut self, rows: Vec<Row>)`**: A builder-style method to add rows.
-   **`Model::update(&mut self, msg: Msg)`**: Handles navigation.
-   **`Model::view(&self)`**: Renders the table.
-   **`Model::selected_row(&self) -> Option<&Row>`**: Gets the currently selected row.

**Usage Example:**
```rust
use bubbletea_widgets::table::{self, Column, Row};
use bubbletea_rs::{Model as BubbleTeaModel, Msg};

struct App {
    table: table::Model,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<bubbletea_rs::Cmd>) {
        let columns = vec![
            Column::new("Rank", 5),
            Column::new("City", 15),
            Column::new("Population", 10),
        ];
        let rows = vec![
            Row::new(vec!["1".into(), "Tokyo".into(), "37M".into()]),
            Row::new(vec!["2".into(), "Delhi".into(), "32M".into()]),
            Row::new(vec!["3".into(), "Shanghai".into(), "28M".into()]),
        ];
        let table_model = table::Model::new(columns).with_rows(rows);
        (Self { table: table_model }, None)
    }

    fn update(&mut self, msg: Msg) -> Option<bubbletea_rs::Cmd> {
        self.table.update(msg)
    }

    fn view(&self) -> String {
        self.table.view()
    }
}
```

---

### File Picker

A component for navigating the filesystem and selecting a file or directory.

**Public API:**
-   **`struct Model`**: The file picker's state.
-   **`fn new() -> Model`**: Creates a new file picker.
-   **`Model::update(&mut self, msg: Msg)`**: Handles navigation and selection.
-   **`Model::view(&self)`**: Renders the file list.
-   **`Model::did_select_file(&self, msg: &Msg) -> (bool, Option<PathBuf>)`**: Checks if a file was selected in the last update.
-   **Fields**: `current_directory: PathBuf`, `path: Option<PathBuf>` (the selected path).

**Usage Example:**
```rust
use bubbletea_widgets::filepicker;
use bubbletea_rs::{Cmd, KeyMsg, Model as BubbleTeaModel, Msg};
use crossterm::event::KeyCode;

struct App {
    file_picker: filepicker::Model,
    selected_file: Option<String>,
}

impl BubbleTeaModel for App {
    fn init() -> (Self, Option<Cmd>) {
        (
            Self {
                file_picker: filepicker::new(),
                selected_file: None,
            },
            None,
        )
    }

    fn update(&mut self, msg: Msg) -> Option<Cmd> {
        // Check if a file was selected
        if let (true, Some(path)) = self.file_picker.did_select_file(&msg) {
            self.selected_file = Some(path.to_string_lossy().to_string());
            return Some(bubbletea_rs::quit());
        }

        self.file_picker.update(msg)
    }

    fn view(&self) -> String {
        if let Some(file) = &self.selected_file {
            return format!("You picked: {}", file);
        }
        format!("Pick a file:\n{}", self.file_picker.view())
    }
}
```

---

### Cursor

The `cursor` is a low-level component, typically used inside other components like `TextInput` and `TextArea`. It manages the visual state of the text cursor, including its shape, style, and blinking.

**Public API:**
-   **`struct Model`**: The cursor's state.
-   **`fn new() -> Model`**: Creates a new cursor.
-   **`enum Mode`**: `Blink`, `Static`, `Hide`.
-   **`Model::set_mode(&mut self, mode: Mode) -> Option<Cmd>`**: Changes the cursor's behavior.
-   **`Model::focus(&mut self) -> Option<Cmd>`**: Activates the cursor and starts blinking.
-   **`Model::blur(&mut self)`**: Deactivates the cursor.
-   **`Model::set_char(&mut self, s: &str)`**: Sets the character the cursor is currently on.
-   **`Model::update(&mut self, msg: &Msg)`**: Handles blink messages.
-   **`Model::view(&self)`**: Renders the cursor.

**Usage Example (Conceptual):**
This component is not typically used standalone. See the source code for `TextInput` or `TextArea` for integration examples.

```rust
use bubbletea_widgets::cursor;
use lipgloss::{Style, Color};

// Inside a text input component's model:
struct MyInput {
    cursor: cursor::Model,
    text: String,
    cursor_pos: usize,
}

impl MyInput {
    fn view(&self) -> String {
        let mut s = self.text.clone();
        if self.cursor.focused() && self.cursor_pos < s.len() {
            // Get the character under the cursor
            let char_under_cursor = s.chars().nth(self.cursor_pos).unwrap().to_string();
            
            // Set the cursor's character and style
            let mut cursor_view = self.cursor.clone();
            cursor_view.set_char(&char_under_cursor);
            cursor_view.style = Style::new().background(Color::from("205"));

            // Replace the character with the cursor's view
            let (start, end) = s.split_at(self.cursor_pos);
            let (_, end) = end.split_at(1);
            s = format!("{}{}{}", start, cursor_view.view(), end);
        }
        s
    }
}