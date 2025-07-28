use std::process::ExitCode;

use rooibos::components::{ListView, WrappingList};
use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::Render;
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Set, With};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::theme::Style;
use rooibos::tui::widgets::ListItem;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    let runtime = Runtime::initialize(DefaultBackend::auto());
    runtime.run(app).await
}

fn app() -> impl Render {
    let selected = RwSignal::new(Some(0));
    let items = RwSignal::new(WrappingList(vec![
        ListItem::new("Item 1"),
        ListItem::new("Item 2"),
        ListItem::new("Item 3"),
    ]));

    ListView::new()
        .on_item_click(move |i, _| {
            selected.set(Some(i));
        })
        .on_key_down(
            [
                key(keys::DOWN, move |_, _| {
                    let selected_idx = selected.get().unwrap();
                    items.with(|i| {
                        selected.set(i.next_index(selected_idx));
                    });
                }),
                key(keys::UP, move |_, _| {
                    let selected_idx = selected.get().unwrap();
                    items.with(|i| {
                        selected.set(i.prev_index(selected_idx));
                    })
                }),
            ]
            .bind(),
        )
        .highlight_style(Style::new().green())
        .render(selected, items)
}
