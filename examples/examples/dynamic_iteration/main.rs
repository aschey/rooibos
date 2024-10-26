use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::dom::{line, span, text};
use rooibos::keybind::{Bind, map_handler};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, GetUntracked, Set, Update};
use rooibos::reactive::graph::wrappers::read::Signal;
use rooibos::reactive::layout::{block, chars, height};
use rooibos::reactive::{
    Render, col, for_each, height, max_width, mount, padding,
    padding_left, row, wgt, width,
};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::{Block, Paragraph};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let (ids, set_ids) = signal(vec![]);
    let (next_id, set_next_id) = signal(0);

    let remove_id = move |id: i32| {
        set_ids.update(|ids| ids.retain(|i| *i != id));
    };

    let add_counter = move || {
        set_ids.update(|s| s.push(next_id.get_untracked()));
        set_next_id.update(|n| *n += 1);
    };

    col![
        props(max_width!(50.), padding!(1.)),
        row![
            props(height!(3.)),
            Button::new()
                .on_click(add_counter)
                .render(text!("Add Counter")),
        ],
        for_each(
            move || ids.get(),
            |k| *k,
            move |i| counter(i, chars(3.), move || remove_id(i))
        )
    ]
    .on_key_down(map_handler("a", move |_| add_counter()))
    .id("root")
}

fn counter(
    id: i32,
    row_height: Signal<taffy::Dimension>,
    on_remove: impl Fn() + Clone + Send + Sync + 'static,
) -> impl Render {
    let (count, set_count) = signal(0);
    let (border_block, set_block) = signal(Block::bordered().border_set(border::EMPTY));

    let update_count = move |change: i32| set_count.update(|c| *c += change);
    let increase = move || update_count(1);
    let decrease = move || update_count(-1);

    row![
        props(height(row_height), padding_left!(1.), block(border_block)),
        wgt!(
            props(width!(15.)),
            Paragraph::new(line!(
                format!("{id}. "),
                "count: ".bold(),
                span!(count.get()).cyan()
            ))
        )
        .on_click(move |_, _, _| increase())
        .on_right_click(move |_, _, _| decrease())
        .on_key_down(
            [
                map_handler("+", move |_| increase()),
                map_handler("-", move |_| decrease()),
                map_handler("d", move |_| on_remove())
            ]
            .bind()
        )
        .on_focus(move |_, _| {
            set_block.set(Block::bordered());
        })
        .on_blur(move |_, _| {
            set_block.set(Block::bordered().border_set(border::EMPTY));
        })
    ]
}

#[cfg(test)]
mod tests;
