use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::layout::{
    Borders, borders, focus_mode, full, height, margin_x, max_width, overflow_y, scroll,
    vertical_list,
};
use rooibos::reactive::dom::{NodeId, Render, line, span, text};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, GetUntracked, Update};
use rooibos::reactive::{StateProp, col, focus_scope, for_each, row, use_state_prop, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
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
        style(max_width(50), height(full()), overflow_y(scroll())),
        row![
            Button::new()
                .on_click(add_counter)
                .render(text!("Add Counter")),
        ],
        focus_scope!(
            style(focus_mode(vertical_list())),
            for_each(
                move || ids.get(),
                |k| *k,
                move |i| counter(NodeId::new(i.to_string()), move || remove_id(i))
            )
        )
    ]
    .on_key_down(key("a", move |_, _| add_counter()))
    .id("root")
    .focusable(false)
}

fn counter(id: NodeId, on_remove: impl Fn() + Clone + Send + Sync + 'static) -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move |change: i32| set_count.update(|c| *c += change);
    let increase = move || update_count(1);
    let decrease = move || update_count(-1);
    let row_borders = StateProp::new(Borders::all().empty()).focused(|b| b.solid());
    let (row_borders, set_row_state) = use_state_prop(row_borders);

    row![
        style(borders(row_borders)),
        wgt!(
            style(margin_x(1)),
            line!(span!("{id}. "), "count: ".bold(), count.get().cyan())
        )
    ]
    .id(id)
    .on_state_change(set_row_state)
    .on_click(move |_| increase())
    .on_right_click(move |_| decrease())
    .on_key_down(
        [
            key(keys::ENTER, move |_, _| increase()),
            key("+", move |_, _| increase()),
            key("-", move |_, _| decrease()),
            key("d", move |_, _| on_remove()),
        ]
        .bind(),
    )
}

#[cfg(test)]
mod tests;
