use std::process::ExitCode;

use rooibos::dom::{KeyCode, KeyEvent, line, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{Render, execute_with_owner, mount, run_with_executor, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

fn main() -> Result {
    execute_with_owner(async_main)
}

#[tokio::main(flavor = "current_thread")]
async fn async_main() -> Result {
    run_with_executor(async {
        mount(app);
        let runtime = Runtime::initialize(CrosstermBackend::stdout());
        runtime.run().await
    })
    .await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    };

    wgt!(line!("count: ".bold(), span!(count.get()).cyan())).on_key_down(key_down)
}
