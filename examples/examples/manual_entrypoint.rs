use std::process::ExitCode;

use rooibos::keybind::{key, keys};
use rooibos::reactive::dom::{Render, line};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{execute_with_owner, run_with_executor, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

fn main() -> Result {
    execute_with_owner(async_main)
}

#[tokio::main(flavor = "current_thread")]
async fn async_main() -> Result {
    run_with_executor(async {
        let runtime = Runtime::initialize(DefaultBackend::auto().await?);
        runtime.run(app).await
    })
    .await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    wgt!(line!("count: ".bold(), count.get()).cyan()) //
        .on_key_down(key(keys::ENTER, move |_, _| {
            set_count.update(|c| *c += 1);
        }))
}
