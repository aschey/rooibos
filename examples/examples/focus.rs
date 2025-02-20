use std::process::ExitCode;

use rooibos::keybind::{Bind, key, keys};
use rooibos::reactive::dom::layout::{Borders, borders};
use rooibos::reactive::dom::{Render, clear_focus, focus_next, focus_prev, line, use_focus};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, derive_signal, max_width, padding, row, wgt, width};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
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
            key(keys::UP, move |_, _| {
                focus_prev();
            }),
            key(keys::DOWN, move |_, _| {
                focus_next();
            }),
            key(keys::ESC, move |_, _| {
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

    wgt!(props(borders(Borders::all()), width!(30.)), title.get())
        .id(id)
        .focusable(true)
}
