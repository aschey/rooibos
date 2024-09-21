use rooibos::dom::{KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::layout::{chars, height};
use rooibos::reactive::{Render, col, height, line, max_width, mount, span, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
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

fn counter(row_height: Signal<taffy::Dimension>) -> impl Render {
    let (count, set_count) = signal(0);
    let (block, set_block) = signal(Block::bordered().border_set(border::EMPTY));

    let update_count = move |change: i32| set_count.update(|c| *c += change);
    let increase = move || update_count(1);
    let decrease = move || update_count(-1);

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Up {
            increase();
        }
        if key_event.code == KeyCode::Down {
            decrease();
        }
    };

    wgt![
        props(height(row_height)),
        Paragraph::new(line!("count: ".bold().reset(), span!(count.get()).cyan()))
            .block(block.get())
    ]
    .on_focus(move |_, _| set_block.set(Block::bordered().blue()))
    .on_blur(move |_, _| set_block.set(Block::bordered().border_set(border::EMPTY)))
    .on_key_down(key_down)
    .on_click(move |_, _, _| increase())
    .on_right_click(move |_, _, _| decrease())
}

const NUM_COUNTERS: usize = 5;

fn app() -> impl Render {
    col![
        props(height!(15.), max_width!(20.)),
        (0..NUM_COUNTERS)
            .map(|_| counter(chars(3.)))
            .collect::<Vec<_>>()
    ]
}

#[cfg(test)]
mod tests;
