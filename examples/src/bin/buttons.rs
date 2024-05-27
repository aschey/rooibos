use std::error::Error;
use std::io::Stdout;

use rooibos::components::Button;
use rooibos::dom::{col, derive_signal, row, Constrainable, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::text::Text;

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
    col![counter_button(), counter_button()]
}

fn counter_button() -> impl Render {
    let (count, set_count) = signal(0);
    row![
        col![
            Button::new()
                .on_click(move || set_count.update(|c| *c += 1))
                .render(derive_signal!(Text::from(format!("count {}", count.get()))))
        ]
        .length(20)
    ]
    .length(3)
}
