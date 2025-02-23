use std::process::ExitCode;

use rooibos::components::Input;
use rooibos::reactive::dom::layout::{Borders, borders};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line, use_focus};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, derive_signal, padding, wgt, width};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, max_viewport_width};
use rooibos::terminal::DefaultBackend;
use rooibos::tui::style::Stylize;
use taffy::LengthPercentage;

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
        props(
            padding!(1),
            width!(100%),
            borders(derive_signal!(if focused.get() {
                Borders::all().title("Input").blue()
            } else {
                Borders::all().title("Input")
            }))
        ),
        Input::default()
            .id(id)
            .padding_bottom(LengthPercentage::Length(1.))
            .placeholder_text("Enter some text")
            .on_submit(move |_| {
                textarea.delete_line();
            })
            .render(textarea),
        wgt!(line!("You typed: ", text.get().bold()))
    ]
}
