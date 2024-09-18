use rooibos::dom::{KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::{signal, RwSignal};
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::layout::{chars, height};
use rooibos::reactive::{col, height, line, max_width, mount, span, wgt, Render};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::{Block, Paragraph};

type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

fn counter(id: i32, row_height: Signal<taffy::Dimension>) -> impl Render {
    let (count, set_count) = signal(0);

    let block = RwSignal::new(Block::bordered().border_set(border::EMPTY));

    let update_count = move |change: i32| set_count.update(|c| *c += change);

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Up {
            update_count(1);
        }
        if key_event.code == KeyCode::Down {
            update_count(-1);
        }
    };

    wgt![
        props(height(row_height)),
        Paragraph::new(line!("count: ".bold().reset(), span!(count.get()).cyan()))
            .block(block.get())
    ]
    .on_focus(move |_, _| block.set(Block::bordered().blue()))
    .on_blur(move |_, _| block.set(Block::bordered().border_set(border::EMPTY)))
    .on_key_down(key_down)
    .on_click(move |_, _, _| update_count(1))
    .id(id.to_string())
}

fn app() -> impl Render {
    col![
        props(height!(15.), max_width!(20.)),
        (0..5).map(|i| counter(i, chars(3.))).collect::<Vec<_>>()
    ]
}

#[cfg(test)]
mod tests;
