use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{
    col, focus_next, focus_prev, row, use_focus, widget_ref, Constrainable, KeyCode, Render,
};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::traits::Get;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{use_keypress, Runtime, RuntimeSettings};
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

fn app() -> impl Render {
    let term_signal = use_keypress();
    Effect::new(move |_| {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Up {
                focus_prev();
            }
            if term_signal.code == KeyCode::Down {
                focus_next();
            }
        }
    });

    row![
        col![
            row![focus_block("item 1")].percentage(50),
            row![focus_block("item 2")].percentage(50)
        ],
        col![
            row![focus_block("item 3")].percentage(50),
            row![focus_block("item 4")].percentage(50)
        ]
    ]
}

fn focus_block(title: &'static str) -> impl Render {
    let (id, focused) = use_focus();

    widget_ref!(
        Paragraph::new(format!("{title} - focused: {}", focused.get())).block(Block::default())
    )
    .id(id)
    .focusable(true)
}
