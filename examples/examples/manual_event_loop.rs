use std::process::ExitCode;

use rooibos::keybind::{key, keys};
use rooibos::reactive::dom::{Render, focus_next, line, render_terminal, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::wgt;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, TickResult, restore_terminal};
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    let mut runtime = Runtime::initialize(DefaultBackend::auto());

    runtime.mount(app);
    let mut terminal = runtime
        .setup_terminal()
        .await
        .map_err(RuntimeError::SetupFailure)?;
    runtime.draw(&mut terminal).await;
    focus_next();

    loop {
        let tick_result = runtime.tick().await?;
        match tick_result {
            TickResult::Redraw => {
                render_terminal(&mut terminal).await?;
            }
            TickResult::Restart => {
                terminal = runtime.setup_terminal().await?;
                render_terminal(&mut terminal).await?;
            }
            TickResult::Exit(payload) => {
                if runtime.should_exit(payload.clone()).await {
                    runtime.handle_exit(&mut terminal).await?;
                    restore_terminal()?;
                    if let Some(e) = payload.error() {
                        return Err(RuntimeError::UserDefined(e.clone()));
                    } else {
                        return Ok(payload.code().as_exit_code().unwrap_or(ExitCode::FAILURE));
                    }
                }
            }
            TickResult::Command(command) => {
                runtime
                    .handle_terminal_command(command, &mut terminal)
                    .await?;
            }
            TickResult::Continue => {}
        }
    }
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
        .on_key_down(key(keys::ENTER, move |_, _| {
            update_count();
        }))
        .on_click(move |_| update_count())
}
