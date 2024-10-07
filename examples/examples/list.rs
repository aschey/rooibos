use std::process::ExitCode;

use rooibos::components::{ListView, WrappingList};
use rooibos::dom::{KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::RwSignal;
use rooibos::reactive::graph::traits::{Get, Set, With};
use rooibos::reactive::{Render, mount};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::ListItem;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    let selected = RwSignal::new(Some(0));
    let items = RwSignal::new(WrappingList(vec![
        ListItem::new("Item 1"),
        ListItem::new("Item 2"),
        ListItem::new("Item 3"),
    ]));

    let on_key_down = move |key_event: KeyEvent, _, _| {
        let selected_idx = selected.get().unwrap();
        match key_event.code {
            KeyCode::Down => {
                items.with(|i| {
                    selected.set(i.next_index(selected_idx));
                });
            }
            KeyCode::Up => {
                items.with(|i| {
                    selected.set(i.prev_index(selected_idx));
                });
            }
            _ => {}
        }
    };
    ListView::new()
        .on_item_click(move |i, _| {
            selected.set(Some(i));
        })
        .on_key_down(on_key_down)
        .highlight_style(Style::new().green())
        .render(selected, items)
}
