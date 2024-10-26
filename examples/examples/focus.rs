use std::process::ExitCode;

use rooibos::dom::{clear_focus, focus_next, focus_prev, line};
use rooibos::keybind::{Bind, map_handler};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{
    Render, col, derive_signal, height, max_width, mount, padding, row, use_focus, wgt,
};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use rooibos::tui::widgets::{Block, Paragraph};

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main(flavor = "current_thread")]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    row![
        props(padding!(1.)),
        col![
            props(max_width!(60.)),
            focus_block("item 1"),
            focus_block("item 2")
        ],
        col![
            props(max_width!(60.)),
            focus_block("item 3"),
            focus_block("item 4")
        ]
    ]
    .on_key_down(
        [
            map_handler("<Up>", move |_| {
                focus_prev();
            }),
            map_handler("<Down>", move |_| {
                focus_next();
            }),
            map_handler("<Esc>", move |_| {
                clear_focus();
            }),
        ]
        .bind(),
    )
}

fn focus_block(title: &'static str) -> impl Render {
    let (id, focused) = use_focus();

    let title = derive_signal!(if focused.get() {
        line!(title, " - ", "focused".green())
    } else {
        line!(title)
    });

    wgt!(
        props(height!(3.), max_width!(30.)),
        Paragraph::new(title.get())
            .centered()
            .block(Block::bordered())
    )
    .id(id)
    .focusable(true)
}
