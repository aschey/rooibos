use std::error::Error;

use rooibos::components::Button;
use rooibos::dom::{col, derive_signal, length, line, row, span, Constrainable, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize_with_settings(
        RuntimeSettings::default(),
        CrosstermBackend::stdout(),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    col![counter_button(), counter_button()]
}

fn counter_button() -> impl Render {
    let (count, set_count) = signal(0);
    row![
        props(length(3)),
        Button::new()
            .length(20)
            .on_click(move || set_count.update(|c| *c += 1))
            .render(derive_signal!(line!("count: ", span!(count.get())).into()))
    ]
}
#[cfg(test)]
mod tests;
