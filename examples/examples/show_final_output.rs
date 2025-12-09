use std::process::ExitCode;

use rooibos::components::Show;
use rooibos::keybind::{key, keys};
use rooibos::reactive::dom::{Render, after_render, line};
use rooibos::reactive::graph::signal::{RwSignal, signal};
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::{row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{ControlFlow, Runtime, RuntimeSettings, before_exit, exit};
use rooibos::terminal::DefaultBackend;
use rooibos::terminal::termina::TerminalSettings;
use rooibos::tui::Viewport;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize_with(
        RuntimeSettings::default().viewport(Viewport::Inline(1)),
        DefaultBackend::new(TerminalSettings::stdout()?.alternate_screen(false)).await,
    )
    .run(app)
    .await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);
    let is_exiting = RwSignal::new(false);

    before_exit(move |_| {
        if !is_exiting.get() {
            is_exiting.set(true);
            return ControlFlow::Prevent;
        }
        ControlFlow::Allow
    });

    let update_count = move || set_count.update(|c| *c += 1);

    row![
        Show::new()
            .fallback(move || {
                after_render(exit);
                wgt!(line!("final count was ", count.get()))
            })
            .render(
                move || !is_exiting.get(),
                move || {
                    wgt!(line!("count ", count.get()))
                        .on_key_down(key(keys::ENTER, move |_, _| {
                            update_count();
                        }))
                        .on_click(move |_| update_count())
                }
            )
    ]
}
