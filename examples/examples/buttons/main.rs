use rooibos::components::Button;
use rooibos::dom::layout::chars;
use rooibos::dom::{col, derive_signal, height, line, row, span, Render, UpdateLayoutProps};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::Runtime;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    col![counter_button(), counter_button()]
}

fn counter_button() -> impl Render {
    let (count, set_count) = signal(0);
    row![
        props(height!(3.)),
        Button::new()
            .width(chars(20.))
            .on_click(move || set_count.update(|c| *c += 1))
            .render(derive_signal!(line!("count: ", span!(count.get())).into()))
    ]
}
#[cfg(test)]
mod tests;
