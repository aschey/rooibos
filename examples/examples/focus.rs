use std::error::Error;

use rooibos::dom::{flex_col, flex_row, focus_next, focus_prev, use_focus, wgt, KeyCode, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::traits::Get;
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{use_keypress, Runtime};
use rooibos::tui::widgets::{Block, Paragraph};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let term_signal = use_keypress();
    Effect::new(move || {
        if let Some(term_signal) = term_signal.get() {
            if term_signal.code == KeyCode::Up {
                focus_prev();
            }
            if term_signal.code == KeyCode::Down {
                focus_next();
            }
        }
    });

    flex_row![
        flex_col![focus_block("item 1"), focus_block("item 2")],
        flex_col![focus_block("item 2"), focus_block("item 3")],
    ]
}

fn focus_block(title: &'static str) -> impl Render {
    let (id, focused) = use_focus();

    wgt!(Paragraph::new(format!("{title} - focused: {}", focused.get())).block(Block::default()))
        .id(id)
        .focusable(true)
}
