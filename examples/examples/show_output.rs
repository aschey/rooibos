use std::io::Stdout;

use rooibos::components::Show;
use rooibos::dom::{KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::{signal, RwSignal};
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::{after_render, row, wgt, Render};
use rooibos::runtime::backend::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{before_exit, exit, ExitResult, Runtime};
use rooibos::tui::Viewport;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        CrosstermBackend::<Stdout>::new(TerminalSettings::default().viewport(Viewport::Inline(8))),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);
    let exiting = RwSignal::new(false);

    before_exit(move || async move {
        if !exiting.get() {
            exiting.set(true);
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
                wgt!(format!("count {}", count.get()))
                    .on_key_down(key_down)
                    .on_click(move |_, _, _| update_count())
            })
            .render(exiting, move || {
                after_render(move || {
                    exit();
                });
                wgt!(format!("final count was {}", count.get()))
            })
    ]
}
