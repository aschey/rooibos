use std::error::Error;
use std::io::Stdout;

use rooibos::components::{container, Button};
use rooibos::dom::{col, derive_signal, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{run, start, RuntimeSettings, TerminalSettings};
use rooibos::tui::layout::Constraint::*;
use rooibos::tui::text::Text;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    start(RuntimeSettings::default(), app);
    run::<Stdout>(TerminalSettings::default()).await?;
    Ok(())
}

fn app() -> impl Render {
    col![counter_button(), counter_button()]
}

fn counter_button() -> impl Render {
    let (count, set_count) = signal(0);
    container(
        Length(20),
        Length(5),
        Button::new()
            .on_click(move || set_count.update(|c| *c += 1))
            .render(derive_signal!(Text::from(format!("count {}", count.get())))),
    )
}
