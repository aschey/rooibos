use std::error::Error;

use rooibos::reactive::dom::layout::{Borders, borders};
use rooibos::reactive::dom::{Render, render_single_frame};
use rooibos::reactive::wgt;
use rooibos::terminal::{Backend, DefaultBackend};
use rooibos::tui::Viewport;
use rooibos::tui::style::Stylize;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = rooibos::tui::Terminal::with_options(
        DefaultBackend::stdout().create_tui_backend()?,
        rooibos::tui::TerminalOptions {
            viewport: Viewport::Inline(3),
        },
    )?;
    render_single_frame(app, &mut terminal)?;

    Ok(())
}

fn app() -> impl Render {
    wgt!(style(borders(Borders::all().cyan())), "hello!".reset())
}
