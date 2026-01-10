use std::process::ExitCode;

use rooibos::keybind::{key, keys};
use rooibos::reactive::dom::{Render, line};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::wgt;
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto().await?)
        .run(|_| app())
        .await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0u32);

    let update_count = move || set_count.update(|c| *c += 1);

    wgt!(line!("count: ".bold(), count.get().cyan()))
        .on_key_down(key(keys::ENTER, move |_, _| update_count()))
        .on_click(move |_| update_count())
}

#[cfg(test)]
mod tests;
