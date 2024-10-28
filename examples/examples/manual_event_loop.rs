use std::process::ExitCode;

use rooibos::keybind::map_handler;
use rooibos::reactive::dom::{Render, focus_next, line, mount, render_terminal, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::wgt;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, TickResult, restore_terminal};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let mut runtime = Runtime::initialize(CrosstermBackend::stdout());
    let mut terminal = runtime.setup_terminal()?;

    render_terminal(&mut terminal)?;
    focus_next();

    loop {
        let tick_result = runtime.tick().await?;
        match tick_result {
            TickResult::Redraw => {
                render_terminal(&mut terminal)?;
            }
            TickResult::Restart => {
                terminal = runtime.setup_terminal()?;
                render_terminal(&mut terminal)?;
            }
            TickResult::Exit(Ok(code)) => {
                if runtime.should_exit().await {
                    runtime.handle_exit(&mut terminal).await.unwrap();
                    restore_terminal()?;
                    return Ok(code);
                }
            }
            TickResult::Exit(Err(e)) => {
                runtime.handle_exit(&mut terminal).await.unwrap();
                restore_terminal()?;
                return Err(RuntimeError::UserDefined(e));
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

    wgt!(line!("count: ".bold(), span!(count.get()).cyan())).on_key_down(map_handler(
        "<Enter>",
        move |_, _| {
            set_count.update(|c| *c += 1);
        },
    ))
}
