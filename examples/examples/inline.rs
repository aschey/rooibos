use std::collections::VecDeque;
use std::process::ExitCode;
use std::time::Duration;

use rooibos::components::Show;
use rooibos::components::spinner::Spinner;
use rooibos::reactive::dom::layout::padding_left;
use rooibos::reactive::dom::{Render, after_render, line};
use rooibos::reactive::graph::computed::Memo;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update, With as _};
use rooibos::reactive::{col, derive_signal, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, RuntimeSettings, exit, insert_before};
use rooibos::terminal::DefaultBackend;
use rooibos::terminal::crossterm::TerminalSettings;
use rooibos::tui::Viewport;
use rooibos::tui::style::{Style, Stylize};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize_with(
        RuntimeSettings::default().viewport(Viewport::Inline(1)),
        DefaultBackend::auto().settings(TerminalSettings::default().alternate_screen(false)),
    )
    .run(app)
    .await
}

fn app() -> impl Render {
    let (packages, set_packages) = signal(VecDeque::from(vec![
        "tokio",
        "ratatui",
        "leptos",
        "taffy",
        "russh",
        "crossterm",
        "termwiz",
        "termion",
    ]));

    let current_package = derive_signal!(packages.with(|p| p.front().copied().unwrap_or_default()));

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(get_random_delay()).await;
            insert_before(1, line!(" ✓ ".green(), current_package.get())).unwrap();
            set_packages.update(|p| {
                p.pop_front();
            });
        }
    });

    let spinner = Spinner::new()
        .spinner_style(Style::default().cyan())
        .into_span_signal();

    col![
       style(padding_left(1)),
        Show::new()
            .fallback(move || {
                after_render(exit);
                wgt!("Done".bold())
            })
            .render(
                Memo::new(move |_| !current_package.get().is_empty()),
                move || wgt!(line!(
                    spinner.get(),
                    "building ",
                    current_package.get().bold(),
                    "..."
                )),
            )
    ]
}

fn get_random_delay() -> Duration {
    Duration::from_millis(((rand::random::<f32>() + 0.5) * 500.0).round() as u64)
}
