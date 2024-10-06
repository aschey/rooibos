use std::process::ExitCode;

use rooibos::dom::{KeyCode, KeyEvent, focus_next, line, render_terminal, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{Render, mount, wgt};
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

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    };

    wgt!(line!("count: ".bold(), span!(count.get()).cyan())).on_key_down(key_down)
}
