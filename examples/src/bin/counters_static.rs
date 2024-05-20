use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{col, widget_ref, Constrainable, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::{signal, RwSignal};
use rooibos::reactive::traits::{Get, Set, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{start, RuntimeSettings};
use rooibos::tui::layout::Constraint::{self, *};
use rooibos::tui::style::Stylize;
use rooibos::tui::widgets::{Block, Padding, Paragraph};

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

fn counter(id: i32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);
    let default_padding = Padding {
        left: 1,
        top: 1,
        ..Default::default()
    };
    let block = RwSignal::new(Block::default().padding(default_padding));

    let update_count = move |change: i32| set_count.update(|c| *c += change);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Up {
            update_count(1);
        }
        if key_event.code == KeyCode::Down {
            update_count(-1);
        }
    };

    widget_ref!(Paragraph::new(format!("count: {}", count.get())).block(block.get()))
        .constraint(constraint)
        .on_focus(move |_| block.set(Block::bordered().blue()))
        .on_blur(move |_| block.set(Block::default().padding(default_padding)))
        .on_key_down(key_down)
        .on_click(move |_, _| update_count(1))
        .id(id.to_string())
}

fn app() -> impl Render {
    col![{ (0..5).map(|i| counter(i, Length(3))).collect::<Vec<_>>() }]
}
