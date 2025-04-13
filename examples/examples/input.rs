use std::process::ExitCode;

use rooibos::components::Input;
use rooibos::reactive::dom::layout::{Borders, borders, full, padding, width};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line, use_focus};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, derive_signal, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, max_viewport_width};
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto()).run(app).await
}

fn app() -> impl Render {
    max_viewport_width(100).unwrap();

    let textarea = Input::get_ref();
    let (id, focused) = use_focus();
    let text = textarea.text();
    col![
        style(
            padding(1),
            width(full()),
            borders(derive_signal!({
                let borders = Borders::all().title("Input");
                if focused.get() {
                    borders.blue()
                } else {
                    borders
                }
            }))
        ),
        Input::default()
            .id(id)
            .padding_bottom(1)
            .placeholder_text("Enter some text")
            .on_submit(move |_| {
                textarea.delete_line();
            })
            .render(textarea),
        wgt!(line!("You typed: ", text.get().bold()))
    ]
}
