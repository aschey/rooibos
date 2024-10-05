<div align="center">

![Rooibos Logo](./assets/logo_text.svg)

**TUI apps that run anywhere**

![license](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)
[![CI](https://github.com/aschey/rooibos/actions/workflows/test.yml/badge.svg)](https://github.com/aschey/rooibos/actions/workflows/test.yml)
![GitHub repo size](https://img.shields.io/github/repo-size/aschey/rooibos)
![Lines of Code](https://aschey.tech/tokei/github/aschey/rooibos)

</div>

**NOTE: This project is currently in a pre-alpha state and should not be used for anything beyond experimentation yet.**

# Intro

Rooibos is an application framework for creating TUI ([text-based user interface](https://en.wikipedia.org/wiki/Text-based_user_interface)) apps that can run in a variety of different environments - in the terminal, web, desktop and more.
It uses a [signal-based](https://github.com/leptos-rs/leptos/tree/main/reactive_graph) reactivity model to build declarative user interfaces.

# Example

```rust
use std::process::ExitCode;

use rooibos::dom::{line, span, KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{mount, wgt, Render};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
        .on_key_down(key_down)
        .on_click(move |_, _, _| update_count())
}
```
