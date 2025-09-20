use std::error::Error;

use rooibos::reactive::dom::layout::{Borders, borders};
use rooibos::reactive::dom::{Render, render_single_frame};
use rooibos::reactive::wgt;
use rooibos::terminal::termina::TerminalSettings;
use rooibos::terminal::{Backend, DefaultBackend};
use rooibos::theme::Stylize;
use rooibos::tui::Viewport;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let backend = DefaultBackend::new(TerminalSettings::stdout()?.alternate_screen(false)).await;
    let mut terminal = rooibos::tui::Terminal::with_options(
        backend.create_tui_backend()?,
        rooibos::tui::TerminalOptions {
            viewport: Viewport::Inline(3),
        },
    )?;
    render_single_frame(app, &mut terminal)?;
    backend.restore_terminal()?;

    Ok(())
}

fn app() -> impl Render {
    wgt!(style(borders(Borders::all().cyan())), "hello!".reset())
}
