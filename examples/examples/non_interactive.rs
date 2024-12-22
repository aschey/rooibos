use std::error::Error;
use std::io::stdout;

use rooibos::reactive::dom::layout::{Borders, borders};
use rooibos::reactive::dom::{Render, render_single_frame};
use rooibos::reactive::wgt;
use rooibos::tui::Viewport;
use rooibos::tui::backend::CrosstermBackend;
use rooibos::tui::style::Stylize;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = rooibos::tui::Terminal::with_options(
        CrosstermBackend::new(stdout()),
        rooibos::tui::TerminalOptions {
            viewport: Viewport::Inline(3),
        },
    )?;
    render_single_frame(app, &mut terminal)?;

    Ok(())
}

fn app() -> impl Render {
    wgt!(props(borders(Borders::all().cyan())), "hello!".reset())
}
