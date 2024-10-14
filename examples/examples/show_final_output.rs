use std::io::Stdout;
use std::process::ExitCode;

use rooibos::components::Show;
use rooibos::dom::{KeyCode, KeyEventProps};
use rooibos::reactive::graph::signal::{RwSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::{Render, after_render, derive_signal, mount, row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{ExitResult, Runtime, RuntimeSettings, before_exit, exit};
use rooibos::terminal::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::tui::Viewport;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize_with_settings(
        RuntimeSettings::default().viewport(Viewport::Inline(1)),
        CrosstermBackend::new(TerminalSettings::<Stdout>::new().alternate_screen(false)),
    );
    runtime.run().await
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

    let key_down = move |props: KeyEventProps| {
        if props.event.code == KeyCode::Enter {
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
