use std::process::ExitCode;

use rooibos::components::Input;
use rooibos::reactive::dom::layout::{Borders, borders};
use rooibos::reactive::dom::{Render, UpdateLayoutProps, line, use_focus};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, derive_signal, max_width, padding, wgt, width};
use rooibos::runtime::Runtime;
use rooibos::runtime::error::RuntimeError;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;
use taffy::LengthPercentage;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(CrosstermBackend::stdout())
        .run(app)
        .await
}

fn app() -> impl Render {
    let textarea = Input::get_ref();
    let (id, focused) = use_focus();
    let text = textarea.text();
    col![
        props(
            padding!(1.),
            width!(100.%),
            max_width!(100.),
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
