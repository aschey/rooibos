use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::layout::{Borders, borders};
use rooibos::reactive::dom::{Render, line, span, text};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, GetUntracked, Set, Update};
use rooibos::reactive::{col, for_each, height, margin_x, max_width, row, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(CrosstermBackend::stdout())
        .run(app)
        .await
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
        props(max_width!(50.)),
        row![
            props(height!(3.)),
            Button::new()
                .on_click(add_counter)
                .render(text!("Add Counter")),
        ],
        for_each(
            move || ids.get(),
            |k| *k,
            move |i| counter(i, move || remove_id(i))
        )
    ]
    .on_key_down(key("a", move |_, _| add_counter()))
    .id("root")
}

fn counter(id: i32, on_remove: impl Fn() + Clone + Send + Sync + 'static) -> impl Render {
    let (count, set_count) = signal(0);
    let (border_block, set_block) = signal(Borders::all().empty());

    let update_count = move |change: i32| set_count.update(|c| *c += change);
    let increase = move || update_count(1);
    let decrease = move || update_count(-1);

    row![
        props(borders(border_block)),
        wgt!(
            props(margin_x!(1.)),
            line!(
                format!("{id}. "),
                "count: ".bold(),
                span!(count.get()).cyan()
            )
        )
        .on_click(move |_| increase())
        .on_right_click(move |_| decrease())
        .on_key_down(
            [
                key(keys::ENTER, move |_, _| increase()),
                key("+", move |_, _| increase()),
                key("-", move |_, _| decrease()),
                key("d", move |_, _| on_remove())
            ]
            .bind()
        )
        .on_focus(move |_, _| {
            set_block.set(Borders::all());
        })
        .on_blur(move |_, _| {
            set_block.set(Borders::all().empty());
        })
    ]
}

#[cfg(test)]
mod tests;
