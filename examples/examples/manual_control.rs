use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{focus_next, render_dom, unmount, widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings, TickResult};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let mut runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    let mut terminal = runtime.setup_terminal()?;

    terminal.draw(|f| render_dom(f.buffer_mut()))?;
    focus_next();
    loop {
        let tick_result = runtime.tick().await;
        match tick_result {
            TickResult::Redraw => {
                terminal.draw(|f| render_dom(f.buffer_mut()))?;
            }
            TickResult::Restart => {
                terminal = runtime.setup_terminal()?;
                terminal.draw(|f| render_dom(f.buffer_mut()))?;
            }
            TickResult::Exit => {
                terminal.clear()?;
                unmount();
                return Ok(());
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

    widget_ref!(format!("count {}", count.get())).on_key_down(key_down)
}
