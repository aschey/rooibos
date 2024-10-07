use std::process::ExitCode;
use std::time::Duration;

use futures_cancel::FutureExt;
use rooibos::components::either_of::Either;
use rooibos::dom::{line, span};
use rooibos::reactive::graph::signal::{RwSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{Render, col, mount, padding, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, ServiceContext, spawn_service, wasm_compat};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let (elapsed, set_elapsed) = signal(0);
    let cancelled = RwSignal::new(false);
    spawn_service(
        ("timer_service", move |context: ServiceContext| async move {
            let start = wasm_compat::now();
            while wasm_compat::sleep(Duration::from_secs(1))
                .cancel_with(context.cancelled())
                .await
                .is_ok()
            {
                set_elapsed.set(((wasm_compat::now() - start) / 1000.0) as u32);
            }
            cancelled.set(true);
            wasm_compat::sleep(Duration::from_millis(500)).await;

            Ok(())
        }),
    );

    col![props(padding!(1.)), move || {
        if cancelled.get() {
            Either::Left(wgt!("App is shutting down..."))
        } else {
            Either::Right(wgt!(line!(
                "timer: ".bold(),
                span!(elapsed.get().to_string()).cyan()
            )))
        }
    }]
}
