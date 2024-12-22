use std::process::ExitCode;

use rooibos::keybind::{keys, map_handler};
use rooibos::reactive::dom::{Render, line, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::wgt;
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(CrosstermBackend::stdout())
        .run(app)
        .await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);
    let key_handler = map_handler(keys::ENTER, move |_, _| update_count());

    wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
        .on_key_down(key_handler)
        .on_click(move |_| update_count())
}

#[cfg(test)]
mod tests;
