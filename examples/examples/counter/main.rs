use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{line, widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::style::Stylize;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    widget_ref!(line!("count: ".bold(), count.get().to_string().cyan()))
        .on_key_down(key_down)
        .on_click(move |_, _| update_count())
}

#[cfg(test)]
#[path = "./tests.rs"]
mod tests;
