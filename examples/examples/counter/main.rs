use std::process::ExitCode;

use rooibos::dom::{line, span};
use rooibos::keybind::map_handler;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{Render, mount, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
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
    let key_handler = map_handler("<Enter>", move |_, _| update_count());

    wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
        .on_key_down(key_handler)
        .on_click(move |_| update_count())
}

#[cfg(test)]
mod tests;
