use std::fs::File;
use std::process::ExitCode;
use std::time::Duration;

use rooibos::reactive::dom::Render;
use rooibos::reactive::wgt;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::wasm_compat::spawn_local;
use rooibos::runtime::{Runtime, wasm_compat};
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;
use tracing::{Level, info};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    if let Ok(file) = File::options().write(true).open("./example.log") {
        // Truncate the output from the last run, but don't remove it.
        // That way you don't need to restart `tail -f example.log` if it's already running.
        file.set_len(0)?;
    }
    let appender = tracing_appender::rolling::never(".", "example.log");
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .with_writer(non_blocking_appender);
    subscriber.init();

    let runtime = Runtime::initialize(DefaultBackend::auto().await?);
    runtime.run(|_| app()).await
}

fn app() -> impl Render {
    spawn_local(async move {
        loop {
            info!("info log!");
            wasm_compat::sleep(Duration::from_secs(1)).await;
        }
    });

    wgt!(rooibos::reactive::dom::line!(
        "run ",
        "tail -f debug.log".bold().cyan(),
        " to see the output"
    ))
}
