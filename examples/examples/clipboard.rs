use std::process::ExitCode;

use rooibos::components::{
    Button, Input, Notification, Notifications, Notifier, provide_notifications,
};
use rooibos::reactive::dom::{Render, line, text};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::{col, margin, padding, padding_right, row, wgt};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, set_clipboard};
use rooibos::terminal::ClipboardKind;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    Runtime::initialize(CrosstermBackend::stdout())
        .run(app)
        .await
}

fn app() -> impl Render {
    provide_notifications();
    let notifier = Notifier::new();
    let textarea = Input::get_ref();

    let text = textarea.text();

    col![
        props(padding!(1.)),
        row![
            props(margin!(1.)),
            wgt!(props(padding_right!(1.)), "Input:".bold().cyan()),
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
