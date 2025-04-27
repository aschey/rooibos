use std::process::ExitCode;

use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::layout::{
    Borders, borders, focus_mode, max_width, padding, vertical_list,
};
use rooibos::reactive::dom::{Render, line};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Set, Update};
use rooibos::reactive::{col, focus_scope, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn counter() -> impl Render {
    let (count, set_count) = signal(0);
    let (border_block, set_border_block) = signal(Borders::all().empty());

    let update_count = move |change: i32| set_count.update(|c| *c += change);
    let increase = move || update_count(1);
    let decrease = move || update_count(-1);

    wgt![
        style(borders(border_block)),
        line!("count: ".bold().reset(), count.get().cyan())
    ]
    .on_focus(move |_, _| set_border_block.set(Borders::all().blue()))
    .on_blur(move |_, _| set_border_block.set(Borders::all().empty()))
    .on_key_down(
        [
            key("+", move |_, _| {
                increase();
            }),
            key("-", move |_, _| {
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
        style(max_width(20), padding(1)),
        focus_scope!(
            style(focus_mode(vertical_list())),
            (0..NUM_COUNTERS).map(|_| counter()).collect::<Vec<_>>()
        )
    ]
}

#[cfg(test)]
mod tests;
