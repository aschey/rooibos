use std::error::Error;

use rooibos::components::{ListView, WrappingList};
use rooibos::dom::{EventData, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Set, With};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::Runtime;
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::ListItem;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let selected = RwSignal::new(Some(0));
    let items = RwSignal::new(WrappingList(vec![
        ListItem::new("Item 1"),
        ListItem::new("Item 2"),
        ListItem::new("Item 3"),
    ]));

    let on_key_down = move |key_event: KeyEvent, _: EventData| {
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
