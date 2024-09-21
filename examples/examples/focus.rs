use rooibos::dom::{KeyCode, focus_next, focus_prev};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{Render, col, mount, row, use_focus, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, use_keypress};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::{Block, Paragraph};
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
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

    row![col![focus_block("item 1"), focus_block("item 2")], col![
        focus_block("item 2"),
        focus_block("item 3")
    ],]
}

fn focus_block(title: &'static str) -> impl Render {
    let (id, focused) = use_focus();

    wgt!(Paragraph::new(format!("{title} - focused: {}", focused.get())).block(Block::default()))
        .id(id)
        .focusable(true)
}
