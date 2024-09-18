use rooibos::dom::{focus_next, render_dom, KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{mount, wgt, Render};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{restore_terminal, Runtime, TickResult};
use rooibos::terminal::crossterm::CrosstermBackend;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let mut runtime = Runtime::initialize(CrosstermBackend::stdout());
    let mut terminal = runtime.setup_terminal()?;

    terminal.draw(|f| render_dom(f.buffer_mut()))?;
    focus_next();
    loop {
        let tick_result = runtime.tick().await?;
        match tick_result {
            TickResult::Redraw => {
                terminal.draw(|f| render_dom(f.buffer_mut()))?;
            }
            TickResult::Restart => {
                terminal = runtime.setup_terminal()?;
                terminal.draw(|f| render_dom(f.buffer_mut()))?;
            }
            TickResult::Exit => {
                if runtime.should_exit().await {
                    runtime.handle_exit(&mut terminal).await.unwrap();
                    restore_terminal()?;
                    return Ok(());
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

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    };

    wgt!(format!("count {}", count.get())).on_key_down(key_down)
}
