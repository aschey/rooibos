use std::error::Error;
use std::io;

use rooibos::dom::{col, widget_ref, Constrainable, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{run, start, RuntimeSettings, TerminalSettings};
use rooibos::tui::layout::Constraint;
use rooibos::tui::layout::Constraint::*;
use rooibos::tui::widgets::Block;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    start(RuntimeSettings::default(), app);
    run::<io::Stdout>(TerminalSettings::default()).await?;
    Ok(())
}

fn counter(id: u32, constraint: Constraint) -> impl Render {
    let (count, set_count) = signal(id);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Up {
            set_count.update(|c| *c += 1);
        }
        if key_event.code == KeyCode::Down {
            set_count.update(|c| *c -= 1);
        }
    };

    widget_ref!(Block::new().title(format!("count: {}", count.get())))
        .id(id.to_string())
        .focusable(true)
        .constraint(constraint)
        .on_key_down(key_down)
}

fn app() -> impl Render {
    col![{ (0..5).map(|i| counter(i, Length(2))).collect::<Vec<_>>() }]
}
