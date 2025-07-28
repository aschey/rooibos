use std::process::ExitCode;

use rooibos::components::Button;
use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::layout::{
    Borders, borders, full, half, height, margin_x, max_width, min_height, overflow_y, scroll,
};
use rooibos::reactive::dom::{NodeId, Render, line, text, use_focus_with_id};
use rooibos::reactive::graph::owner::StoredValue;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, GetUntracked, GetValue, Update};
use rooibos::reactive::{col, derive_signal, for_each, row, wgt};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    col![
        style(height(full())),
        row![style(min_height(half())), counter_pane("a")],
        //
        row![style(min_height(half())), counter_pane("b")]
    ]
}

fn counter_pane(prefix: &'static str) -> impl Render {
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
        for_each(
            move || ids.get(),
            |k| *k,
            move |i| counter(NodeId::new(i.to_string() + prefix), move || remove_id(i))
        )
    ]
    .on_key_down(key("a", move |_, _| add_counter()))
}

fn counter(id: NodeId, on_remove: impl Fn() + Clone + Send + Sync + 'static) -> impl Render {
    let (count, set_count) = signal(0);
    let id = StoredValue::new(id);
    let focused = use_focus_with_id(id.get_value());

    let update_count = move |change: i32| set_count.update(|c| *c += change);
    let increase = move || update_count(1);
    let decrease = move || update_count(-1);

    row![
        style(borders(derive_signal!(if focused.get() {
            Borders::all()
        } else {
            Borders::all().empty()
        }))),
        wgt!(
            style(margin_x(1)),
            line!(
                format!("{}. ", id.get_value()),
                "count: ".bold(),
                count.get().cyan()
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
        .id(id.get_value())
    ]
}
