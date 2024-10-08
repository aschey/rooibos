use std::process::ExitCode;

use rooibos::dom::{KeyCode, clear_focus, focus_next, focus_prev, line};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{
    Render, col, derive_signal, height, max_width, mount, padding, row, use_focus, wgt,
};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, use_keypress};
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
    let term_signal = use_keypress();
    Effect::new(move || {
        if let Some(term_signal) = term_signal.get() {
            match term_signal.code {
                KeyCode::Up => {
                    focus_prev();
                }
                KeyCode::Down => {
                    focus_next();
                }
                KeyCode::Esc => {
                    clear_focus();
                }
                _ => {}
            }
        }
    });

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
