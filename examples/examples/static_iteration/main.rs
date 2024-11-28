use std::process::ExitCode;

use rooibos::keybind::{Bind, map_handler};
use rooibos::reactive::dom::layout::{chars, height};
use rooibos::reactive::dom::{Render, line, mount, span};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::{col, height, max_width, padding, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::{Block, Paragraph};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn counter(row_height: Signal<taffy::Dimension>) -> impl Render {
    let (count, set_count) = signal(0);
    let (block, set_block) = signal(Block::bordered().border_set(border::EMPTY));

    let update_count = move |change: i32| set_count.update(|c| *c += change);
    let increase = move || update_count(1);
    let decrease = move || update_count(-1);

    wgt![
        props(height(row_height)),
        line!("count: ".bold().reset(), span!(count.get()).cyan()).block(block.get())
    ]
    .on_focus(move |_, _| set_block.set(Block::bordered().blue()))
    .on_blur(move |_, _| set_block.set(Block::bordered().border_set(border::EMPTY)))
    .on_key_down(
        [
            map_handler("<Up>", move |_, _| {
                increase();
            }),
            map_handler("<Down>", move |_, _| {
                decrease();
            }),
        ]
        .bind(),
    )
    .on_click(move |_| increase())
    .on_right_click(move |_| decrease())
}

const NUM_COUNTERS: usize = 5;

fn app() -> impl Render {
    col![
        props(height!(20.), max_width!(20.), padding!(1.)),
        (0..NUM_COUNTERS)
            .map(|_| counter(chars(3.)))
            .collect::<Vec<_>>()
    ]
}

#[cfg(test)]
mod tests;
