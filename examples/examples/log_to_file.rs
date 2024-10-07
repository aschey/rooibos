use std::fs;
use std::process::ExitCode;
use std::time::Duration;

use rooibos::reactive::{Render, mount, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::wasm_compat::spawn_local;
use rooibos::runtime::{Runtime, wasm_compat};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use tracing::{Level, info};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    // remove the previous output if it exists
    let _ = fs::remove_file("./example.log");
    let appender = tracing_appender::rolling::never(".", "example.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .with_writer(non_blocking_appender);
    subscriber.init();

    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    spawn_local(async move {
        loop {
            info!("info log!");
            wasm_compat::sleep(Duration::from_secs(1)).await;
        }
    });

    wgt!(rooibos::dom::line!(
        "run ",
        "tail -f debug.log".bold().cyan(),
        " to see the output"
    ))
}
