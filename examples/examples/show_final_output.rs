use std::io::Stdout;

use rooibos::components::Show;
use rooibos::dom::{KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::{RwSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::{Render, after_render, derive_signal, mount, row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{ExitResult, Runtime, before_exit, exit};
use rooibos::terminal::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::tui::Viewport;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::<Stdout>::new(
        TerminalSettings::default().viewport(Viewport::Inline(8)),
    ));
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);
    let is_exiting = RwSignal::new(false);

    before_exit(move || async move {
        if !is_exiting.get() {
            is_exiting.set(true);
            return ExitResult::PreventExit;
        }
        ExitResult::Exit
    });

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    row![
        Show::new()
            .fallback(move || {
                after_render(exit);
                wgt!(format!("final count was {}", count.get()))
            })
            .render(derive_signal!(!is_exiting.get()), move || {
                wgt!(format!("count {}", count.get()))
                    .on_key_down(key_down)
                    .on_click(move |_, _, _| update_count())
            })
    ]
}
