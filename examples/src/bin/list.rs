use std::error::Error;
use std::io::Stdout;

use rooibos::components::ListView;
use rooibos::dom::Render;
use rooibos::reactive::signal::RwSignal;
use rooibos::reactive::traits::Set;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{start, RuntimeSettings};
use rooibos::tui::style::{Style, Stylize};
use rooibos::tui::widgets::ListItem;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let handle = start(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    handle.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let selected = RwSignal::new(None);
    ListView::new()
        .on_item_click(move |i, _| {
            selected.set(Some(i));
        })
        .highlight_style(Style::new().green())
        .render(
            selected,
            vec![ListItem::new("Item 1"), ListItem::new("Item 2")],
        )
}
