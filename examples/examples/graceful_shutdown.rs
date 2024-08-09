use std::error::Error;
use std::time::Duration;

use rooibos::components::either_of::Either;
use rooibos::dom::{col, line, span, widget_ref, Render};
use rooibos::reactive::signal::{signal, RwSignal};
use rooibos::reactive::traits::{Get, Set, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{spawn_service, wasm_compat, Runtime, ServiceContext};
use rooibos::tui::style::Stylize;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);
    let cancelled = RwSignal::new(false);
    spawn_service((
        "counter_service",
        move |context: ServiceContext| async move {
            loop {
                tokio::select! {
                    _ = wasm_compat::sleep(Duration::from_secs(1)) => {
                        set_count.update(|c| *c += 1);
                    }
                    _ = context.cancelled() => {
                        cancelled.set(true);
                        wasm_compat::sleep(Duration::from_millis(500)).await;
                        return Ok(())
                    }
                }
            }
        },
    ));

    col![move || {
        if cancelled.get() {
            Either::Left(widget_ref!("App is shutting down..."))
        } else {
            Either::Right(widget_ref!(line!(
                "count: ".bold(),
                span!(count.get()).cyan()
            )))
        }
    }]
}
