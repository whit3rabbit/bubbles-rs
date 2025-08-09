# bubbletea-widgets

[![CI](https://github.com/whit3rabbit/bubbles-rs/workflows/CI/badge.svg)](https://github.com/whit3rabbit/bubbles-rs/actions)

Rust components for building TUIs with [`bubbletea-rs`](https://github.com/whit3rabbit/bubbletea-rs), ported from Charmbracelet's Go
[`bubbles`](https://github.com/charmbracelet/bubbles). This is a Rust implementation of
the original Go code. All credit for the original designs and APIs goes to the
Charm team and Go community.

## Installation

Add `bubbletea-widgets` to your `Cargo.toml` dependencies. You will also need `bubbletea-rs` and `lipgloss-extras` for a complete TUI application.

```toml
[dependencies]
bubbletea-rs = "0.0.6"
bubbletea-widgets = "0.1.0"
lipgloss-extras = { version = "0.0.7", features = ["full"] }
```

> **Note**: This repository is named `bubbles-rs` for historical reasons, but the package name on crates.io is `bubbletea-widgets`. The original `bubbles-rs` name was already taken by another TUI framework. Always use `bubbletea-widgets` when adding this crate to your dependencies.

## Components

### Spinner

A spinner for indicating an operation is in progress. Includes multiple presets
and option-style configuration.

```rust
use bubbletea_widgets::spinner::{new, with_spinner, with_style, DOT};
use lipgloss::{Style, Color};

let sp = new(&[
    with_spinner(DOT.clone()),
    with_style(Style::new().foreground(Color::from("cyan")))
]);
let frame = sp.view();
```

### Text Input

Single-line input akin to HTML’s `<input type="text">`. Supports unicode, pasting,
in-place scrolling, and customizable key bindings.

```rust
use bubbletea_widgets::textinput::new;

let mut input = new();
input.set_placeholder("Your name…");
input.set_width(30);
let _ = input.focus();
```

### Text Area

Multi-line text input akin to `<textarea>`. Supports unicode, soft-wrapping,
vertical scrolling, and rich styling via Lip Gloss.

```rust
use bubbletea_widgets::textarea;

let mut ta = textarea::new();
ta.set_width(40);
ta.set_height(6);
ta.insert_string("Hello, world!\nThis is bubbletea-widgets.");
let view = ta.view();
```

### Table

Scrollable, navigable tables with headers, selection, and styling.

```rust
use bubbletea_widgets::table::{Model, Column, Row};

let columns = vec![
    Column::new("Name", 20),
    Column::new("Age", 6),
];
let rows = vec![
    Row::new(vec!["Alice".into(), "30".into()]),
    Row::new(vec!["Bob".into(), "25".into()]),
];
let table = Model::new(columns).with_rows(rows);
let _out = table.view();
```

### Progress

Simple, customizable progress meter with optional animation and gradients.

```rust
use bubbletea_widgets::progress::{new, with_width, with_solid_fill};

let mut p = new(&[with_width(30), with_solid_fill("#00ff88".into())]);
let _cmd = p.set_percent(0.4);
let out = p.view();
```

### Paginator

Pagination logic and rendering for dot-style or numeric pagination.

```rust
use bubbletea_widgets::paginator::Model;

let mut p = Model::new();
p.set_per_page(10);
p.set_total_items(95);
let view = p.view(); // e.g., "1/10" or dots depending on type
```

### Viewport

Vertically scrollable viewport for large content; supports key bindings and
horizontal scrolling.

```rust
use bubbletea_widgets::viewport;

let mut vp = viewport::new(80, 20);
vp.set_content("line 1\nline 2\nline 3");
let visible = vp.visible_lines();
```

### List

Customizable, batteries-included list with pagination, fuzzy filtering, spinner,
status messages, and auto-generated help.

```rust
use bubbletea_widgets::list::{Model, DefaultDelegate, Item};

#[derive(Clone)]
struct ItemStr(&'static str);
impl std::fmt::Display for ItemStr { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) } }
impl Item for ItemStr { fn filter_value(&self) -> String { self.0.to_string() } }

let items = vec![ItemStr("foo"), ItemStr("bar")];
let list = Model::new(items, DefaultDelegate::new(), 80, 24);
let _out = list.view();
```

### File Picker

Navigate directories and select files with keyboard navigation and customizable styles.

```rust
use bubbletea_widgets::filepicker::Model;

let (picker, _cmd) = Model::init();
let _out = picker.view();
```

### Timer

Countdown timer with configurable interval and start/stop/toggle commands.

```rust
use bubbletea_widgets::timer::new;
use std::time::Duration;

let timer = new(Duration::from_secs(10));
let cmd = timer.init(); // schedule ticks
let view = timer.view();
```

### Stopwatch

Count-up timer with start/stop/toggle and reset.

```rust
use bubbletea_widgets::stopwatch::new;

let sw = new();
let start_cmd = sw.start();
```

### Help

Horizontal mini help view that auto-generates from your key bindings; supports
single and multi-line modes and truncates gracefully.

```rust
// Help is integrated into components via a KeyMap trait and `help::Model`.
// See `help.rs` and component-specific `KeyMap` implementations.
```

### Key

Non-visual key binding management with help text generation and matching utilities.

```rust
use bubbletea_widgets::key::{new_binding, with_keys_str, with_help, matches};
use bubbletea_rs::KeyMsg;
use crossterm::event::{KeyCode, KeyModifiers};

let save = new_binding(vec![
    with_keys_str(&["ctrl+s", "f2"]),
    with_help("ctrl+s", "save"),
]);
let quit = new_binding(vec![
    with_keys_str(&["ctrl+c", "q"]),
    with_help("ctrl+c", "quit"),
]);

let msg = KeyMsg { key: KeyCode::Char('s'), modifiers: KeyModifiers::CONTROL };
let matched = matches(&msg, &[&save, &quit]);
```

## There’s more where that came from

Community-maintained Bubbles are listed by Charm & Friends: [additional bubbles](https://github.com/charm-and-friends/additional-bubbles).

## Contributing

Issues and PRs welcome. This project aims to mirror the Go API where it makes
sense in Rust, and to keep the codebase clean and idiomatic.

## Attribution

- Original Go project: [`charmbracelet/bubbles`](https://github.com/charmbracelet/bubbles)
- TUI framework inspiration: [`charmbracelet/bubbletea`](https://github.com/charmbracelet/bubbletea)

This crate is a Rust implementation/port. Design, API concepts, and many behaviors
are derived from the Go implementation by Charmbracelet and contributors.

## License

MIT. See `LICENSE`.
