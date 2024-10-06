use std::error::Error;
use std::io::stdout;

use rooibos::dom::render_single_frame;
use rooibos::reactive::layout::block;
use rooibos::reactive::{Render, col, mount, wgt, width};
use rooibos::tui::Viewport;
use rooibos::tui::backend::CrosstermBackend;
use rooibos::tui::style::Stylize;
use rooibos::tui::widgets::Block;

fn main() -> Result<(), Box<dyn Error>> {
    mount(app);
    let mut terminal = rooibos::tui::Terminal::with_options(
        CrosstermBackend::new(stdout()),
        rooibos::tui::TerminalOptions {
            viewport: Viewport::Inline(3),
        },
    )?;
    render_single_frame(&mut terminal)?;

    Ok(())
}

fn app() -> impl Render {
    col![
        props(block(Block::bordered().cyan()), width!(8.)),
        wgt!("hello!".reset())
    ]
}
