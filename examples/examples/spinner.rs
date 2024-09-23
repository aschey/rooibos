use std::process::ExitCode;
use std::time::Duration;

use rooibos::components::spinner::{Spinner, SpinnerDisplay};
use rooibos::dom::span;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::Set;
use rooibos::reactive::{Render, mount};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::wasm_compat::{sleep, spawn_local};
use rooibos::runtime::{Runtime, exit};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Style, Stylize};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let (spinner_display, set_spinner_display) = signal(SpinnerDisplay::Spin);
    let (label, set_label) = signal(span!("loading..."));

    spawn_local(async move {
        sleep(Duration::from_secs(3)).await;
        set_spinner_display.set(SpinnerDisplay::Full);
        set_label.set(span!("Done"));
        sleep(Duration::from_secs(2)).await;
        exit();
    });

    Spinner::new()
        .label(label)
        .spinner_style(Style::new().cyan())
        .display(spinner_display)
        .render()
}
