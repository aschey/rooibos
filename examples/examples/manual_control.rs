use rooibos::dom::{focus_next, render_dom, wgt, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, TickResult};
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let mut runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
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

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    };

    wgt!(format!("count {}", count.get())).on_key_down(key_down)
}
