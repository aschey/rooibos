use std::process::ExitCode;

use rooibos::components::{Button, Input, Notification, Notifications, Notifier};
use rooibos::reactive::dom::layout::{margin, padding, padding_right};
use rooibos::reactive::dom::{Render, line, text};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, set_clipboard};
use rooibos::terminal::{ClipboardKind, DefaultBackend};
use rooibos::theme::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(DefaultBackend::auto().await?).run(app).await
}

fn app() -> impl Render {
    let textarea = Input::get_ref();

    let text = textarea.text();
    let notifier = Notifier::new();
    col![
        style(padding(1)),
        row![
            style(margin(1)),
            wgt!(style(padding_right(1)), "Input:".bold().cyan()),
            Input::default()
                .placeholder_text("Enter some text")
                .render(textarea),
        ],
        Button::new()
            .on_click(move || {
                set_clipboard(text.get(), ClipboardKind::Clipboard).unwrap();
                notifier.notify(Notification::new(line!(
                    "'",
                    text.get().bold().green(),
                    "' ",
                    "copied to clipboard",
                )));
            })
            .render(text!("Copy to clipboard")),
        Notifications::new().render()
    ]
}
