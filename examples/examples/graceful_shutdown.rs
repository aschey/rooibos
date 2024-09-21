use std::time::Duration;

use futures_cancel::FutureExt;
use rooibos::components::either_of::Either;
use rooibos::dom::{line, span};
use rooibos::reactive::graph::signal::{RwSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::{Render, col, mount, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, ServiceContext, spawn_service, wasm_compat};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);
    let cancelled = RwSignal::new(false);
    spawn_service((
        "counter_service",
        move |context: ServiceContext| async move {
            while wasm_compat::sleep(Duration::from_secs(1))
                .cancel_with(context.cancelled())
                .await
                .is_ok()
            {
                set_count.update(|c| *c += 1);
            }
            cancelled.set(true);
            wasm_compat::sleep(Duration::from_millis(500)).await;

            Ok(())
        },
    ));

    col![move || {
        if cancelled.get() {
            Either::Left(wgt!("App is shutting down..."))
        } else {
            Either::Right(wgt!(line!("count: ".bold(), span!(count.get()).cyan())))
        }
    }]
}
