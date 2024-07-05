use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{col, row, widget_ref, Constrainable, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::{signal, RwSignal};
use rooibos::reactive::traits::{Get, Set, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::layout::Constraint::{self, *};
use rooibos::tui::style::Stylize;
use rooibos::tui::symbols::border;
use rooibos::tui::widgets::{Block, Paragraph};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn counter(id: i32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(0);

    let block = RwSignal::new(Block::bordered().border_set(border::EMPTY));

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
        .on_blur(move |_| block.set(Block::bordered().border_set(border::EMPTY)))
        .on_key_down(key_down)
        .on_click(move |_, _| update_count(1))
        .id(id.to_string())
}

fn app() -> impl Render {
    row![col![(0..5).map(|i| counter(i, Length(3))).collect::<Vec<_>>()].length(15)]
}

#[cfg(test)]
mod tests;
