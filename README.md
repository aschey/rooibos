<div align="center">

![Rooibos Logo](./assets/logo_text.svg)

**TUI apps that run anywhere**

![license](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)
[![CI](https://github.com/aschey/rooibos/actions/workflows/test.yml/badge.svg)](https://github.com/aschey/rooibos/actions/workflows/test.yml)
![GitHub repo size](https://img.shields.io/github/repo-size/aschey/rooibos)
![Lines of Code](https://aschey.tech/tokei/github/aschey/rooibos)

</div>

> [!WARNING]
> This project is currently in a pre-alpha state and should not be used for
> anything beyond experimentation yet. There's a high level todo list for the
> initial release
> [here](https://github.com/aschey/rooibos/issues?q=is%3Aissue+is%3Aopen+label%3Aalpha).

## Intro

Rooibos is an application framework for creating TUI
([text-based user interface](https://en.wikipedia.org/wiki/Text-based_user_interface))
apps that can run in a variety of different environments - in the terminal, web,
desktop and more. It uses [Leptos'](https://github.com/leptos-rs/leptos)
[signal-based reactivity model](https://github.com/leptos-rs/leptos/tree/main/reactive_graph)
to create declarative user interfaces. Elements are rendered to the screen using
[Ratatui](https://docs.rs/ratatui/latest/ratatui/) widgets.

![counter](./examples/examples/counter/counter.gif)

The example above can be written using the following code:

```rust,no_run
use std::process::ExitCode;

use rooibos::keybind::{key, keys};
use rooibos::reactive::dom::{Render, line, mount, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::wgt;
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    wgt!(line!("count: ".bold(), count.get().cyan()))
        .on_key_down(key(keys::ENTER, move |_, _| update_count()))
        .on_click(move |_| update_count())
}
```

## Feature Overview

- Declarative and reactive style for building UIs
- Easy-to-use event handlers with mouse support
- Compatibility with Ratatui and its widget ecosystem
- Flexbox and grid layouts powered by
  [taffy](https://docs.rs/taffy/latest/taffy/)
- Render your applications on a variety of platforms with with minimal setup
- First-class async support
- Supports full screen and inline apps

## Demo

[source](./examples/examples/todos_api/main.rs)

![demo](./examples/examples/todos_api/todos_api.gif)

## Signals

Signals are special variables that will trigger their subscribers to re-run
anytime they are updated.

```rust,no_run
use std::process::ExitCode;

use rooibos::keybind::{key, keys};
use rooibos::reactive::dom::{Render, line, mount, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{col, derive_signal, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);
    // Will automatically update anytime `count` is updated.
    let doubled_count = derive_signal!(count.get() * 2);
    let update_count = move || set_count.update(|c| *c += 1);

    col![
        // Reading a signal inside a widget will cause the widget to re-render
        // when the signal updates.
        wgt!(line!("count: ".bold(), count.get().cyan()))
            .on_key_down(key(keys::ENTER, move |_, _| update_count()))
            .on_click(move |_| update_count()),
        wgt!(format!("doubled count: {}", doubled_count.get()))
    ]
}
```

## Components and Layout

Rooibos applications are built using a functional component model that will feel
familiar if you've used a Javascript framework such as React or SolidJS, or one
of the Rust-based frameworks like Leptos or Dioxus. Anything that returns
`impl Render` can be added to the DOM.

Layout properties can be added using the special `style()` keyword.

```rust,no_run
use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::reactive::dom::layout::{height, padding, padding_right, width};
use rooibos::reactive::dom::{Render, mount, text};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{col, derive_signal, row, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::{Color, Stylize};
use rooibos::tui::text::Span;
use rooibos::tui::widgets::Paragraph;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    row![
        // `style()` is a special keyword that sets the layout properties on a widget
        // or layout node.
        style(padding(1)),
        col![
            style(width(20), padding_right(2)),
            button("bigger".bold()),
            button("smaller".bold())
        ]
    ]
}

// Simple components can be written as functions, while complex ones with optional arguments,
// such as `Button`, may be written as structs.
fn button(title: Span<'static>) -> impl Render {
    row![style(height(3)), Button::new().render(text!(title))]
}
```

## Async and Multithreaded Reactivity

The reactivity model is fully `Send + Sync`, allowing you to update the UI from
any thread or async task.

> [!NOTE]
> Even though the application entrypoint must be async, you don't have to write
> any async code in your application logic if you don't want to. Anything is
> fine as long as you don't block the UI thread for too long!

```rust,no_run
use std::process::ExitCode;
use std::time::Duration;

use rooibos::reactive::dom::{Render, line, mount, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::wgt;
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    tokio::task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            // No need to coordinate dispatching events to the main thread
            // to trigger a re-render.
            update_count();
        }
    });

    wgt!(line!("count: ".bold(), count.get().cyan()))
}
```

## Testing

We provide a first-party package for testing your apps and components at a high
level. The API is inspired by [Testing Library](https://testing-library.com/).

```rust
use rooibos::keybind::{key, keys};
use rooibos::reactive::dom::{self, Render, line, mount, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{KeyCode, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::terminal::DefaultBackend;
use rooibos::tester::{TerminalView, TestHarness};
use rooibos::tui::style::Stylize;

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    wgt!(line!("count: ".bold(), count.get().cyan()))
        .on_key_down(key(keys::ENTER, move |_, _| update_count()))
        .on_click(move |_| update_count())
}

macro_rules! assert_snapshot {
    ($harness:expr) => {
        insta::with_settings!({
            snapshot_path => "./snapshots"
        }, {
            insta::assert_debug_snapshot!($harness.buffer());
        });
    };
}

#[rooibos::test]
async fn test_counter() {
    let mut harness = TestHarness::new_with_settings(
        RuntimeSettings::default().enable_signal_handler(false),
        20,
        10,
    )
    .await;
    harness.mount(app).await;

    assert_snapshot!(harness);

    harness.send_key(KeyCode::Enter);
    harness
        .wait_for(|harness, _| harness.buffer().terminal_view().contains("count: 1"))
        .await
        .unwrap();

    harness.exit().await;
}
```

## Examples

See [examples](./examples/examples).

## Macro Usage

There are a number of macros used in Rooibos, such as the `row!`, `col!`, and
`wgt!` calls. These are all simple wrappers to prevent the user from having to
remember to wrap everything in tuples or add a bunch of extra `move ||` syntax.
If desired, these macros can be replaced with normal function calls at the cost
of verbosity.

Originally, Rooibos was designed to use a
[JSX-like syntax](https://github.com/rs-tml/rstml) (borrowed from Leptos), but
this was abandoned due to the fact that procedural macros are not formatted by
`rustfmt` and getting intellisense to work properly inside of a DSL requires
[some rather esoteric workarounds](https://emi0x7d1.dev/blog/improving-autocompletion-in-your-rust-macros).

This work still exists in the
[macros branch](https://github.com/aschey/rooibos/tree/macros) and may be
revived one day, but only as an optional add-on.

## Architecture

For details on internals and the reason behind certain design decisions, see
[architecture](./ARCHITECTURE.md).

## Backend Support Status

| Crate                                               | Backend                                                        | Type     | Status                            |
| --------------------------------------------------- | -------------------------------------------------------------- | -------- | --------------------------------- |
| [**`rooibos-terminal`**](./crates/rooibos-terminal) | [crossterm](https://docs.rs/crossterm/latest/crossterm/)       | Terminal | Implemented                       |
| [**`rooibos-terminal`**](./crates/rooibos-terminal) | [termion](https://docs.rs/termion/latest/termion/)             | Terminal | Implemented, but missing features |
| [**`rooibos-terminal`**](./crates/rooibos-terminal) | [termwiz](https://docs.rs/termwiz/latest/termwiz/)             | Terminal | Implemented, but missing features |
| [**`rooibos-ssh`**](./crates/rooibos-ssh)           | [russh](https://docs.rs/russh/latest/russh/)                   | SSH      | Implemented                       |
| [**`rooibos-xterm-js`**](./crates/rooibos-xterm-js) | [xterm-js-rs](https://docs.rs/xterm-js-rs/latest/xterm_js_rs/) | Web      | Implemented                       |
| **`rooibos-egui`**                                  | [egui](https://docs.rs/egui/latest/egui/)                      | Desktop  | Planned                           |
| **`rooibos-egui`**                                  | [egui](https://docs.rs/egui/latest/egui/)                      | Mobile   | Planned                           |
| **`rooibos-bevy`**                                  | [bevy](https://docs.rs/bevy/latest/bevy/)                      | Games    | Planned                           |
