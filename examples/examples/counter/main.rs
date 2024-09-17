use rooibos::dom::{line, span, KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{wgt, Render};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::Runtime;
use rooibos::tui::style::Stylize;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    wgt!(line!("count: ".bold(), span!(count.get()).cyan()))
        .on_key_down(key_down)
        .on_click(move |_, _, _| update_count())
}

#[cfg(test)]
mod tests;
