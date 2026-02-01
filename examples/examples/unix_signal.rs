use std::io;
use std::process::ExitCode;

use rooibos::reactive::dom::{Render, RenderAny, line, span, text};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Set};
use rooibos::reactive::{error_view, row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{ControlFlow, Runtime, on_os_signal};
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto().await?)
        .run(|_| boundary())
        .await
}

fn boundary() -> impl Render {
    row!(error_view(app, |e| span!(e).red()))
}

fn app() -> impl RenderAny {
    let (os_signal, set_os_signal) = signal("".to_string());

    on_os_signal(move |signal| {
        set_os_signal.set(format!("{signal:?}"));
        ControlFlow::Prevent
    });

    Ok::<_, io::Error>(wgt!(text!(
        line!("run 'pkill --signal <signal name> unix_signal to trigger the handler"),
        line!(""),
        line!("last control flow signal: ", os_signal.get()),
    )))
}
