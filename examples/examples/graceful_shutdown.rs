use std::process::ExitCode;
use std::time::Duration;

use rooibos::components::either_of::Either;
use rooibos::reactive::dom::layout::padding;
use rooibos::reactive::dom::{Render, line};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{col, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, ServiceContext, spawn_service, wasm_compat};
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;
use tokio_util::future::FutureExt;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto().await?).run(app).await
}

fn app() -> impl Render {
    let (elapsed, set_elapsed) = signal(0);
    let (cancelled, set_cancelled) = signal(false);
    spawn_service(
        ("timer_service", move |context: ServiceContext| async move {
            let start = wasm_compat::now();
            while wasm_compat::sleep(Duration::from_secs(1))
                .with_cancellation_token(context.cancellation_token())
                .await
                .is_some()
            {
                set_elapsed.set(((wasm_compat::now() - start) / 1000.0) as u32);
            }
            set_cancelled.set(true);
            wasm_compat::sleep(Duration::from_millis(500)).await;

            Ok(())
        }),
    );

    col![style(padding(1)), move || {
        if cancelled.get() {
            Either::Left(wgt!("App is shutting down..."))
        } else {
            Either::Right(wgt!(line!("timer: ".bold(), elapsed.get().cyan())))
        }
    }]
}
