use std::error::Error;
use std::io::Stderr;

use rooibos::components::{ListView, WrappingList};
use rooibos::dom::{EventData, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::{Get, Set, With};
use rooibos::runtime::backend::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::runtime::{exit, Runtime};
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::ListItem;
use rooibos::tui::Viewport;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        CrosstermBackend::new(
            TerminalSettings::<Stderr>::new()
                .alternate_screen(false)
                .viewport(Viewport::Inline(3)),
        ),
        app,
    );
    runtime.run().await?;

    Ok(())
}

fn app() -> impl Render {
    let selected = RwSignal::new(Some(0));
    let item_text = ["Item 1", "Item 2", "Item 3"];
    let items = RwSignal::new(WrappingList(
        item_text.iter().map(|t| ListItem::new(*t)).collect(),
    ));

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
            KeyCode::Enter => {
                println!("{}", item_text[selected_idx]);
                exit();
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
