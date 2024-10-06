use std::process::ExitCode;

use rooibos::components::{
    Button, Input, Notification, Notifications, Notifier, provide_notifications,
};
use rooibos::dom::{line, text};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::layout::chars;
use rooibos::reactive::{
    Render, UpdateLayoutProps, col, height, margin, mount, padding, row, wgt, width,
};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{Runtime, set_clipboard};
use rooibos::terminal::ClipboardKind;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result = std::result::Result<ExitCode, RuntimeError>;

#[rooibos::main]
async fn main() -> Result {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await
}

fn app() -> impl Render {
    provide_notifications();
    let notifier = Notifier::new();
    let textarea = Input::get_ref();

    let text = textarea.text();

    col![
        props(padding!(1.)),
        row![
            props(height!(1.), margin!(1.)),
            wgt!(props(width!(7.)), "Input:".bold().cyan()),
            Input::default()
                .placeholder_text("Enter some text")
                .height(chars(1.))
                .render(textarea),
        ],
        Button::new()
            .width(chars(25.))
            .height(chars(3.))
            .on_click(move || {
                set_clipboard(text.get(), ClipboardKind::Clipboard).unwrap();
                notifier.notify(Notification::new(line!(
                    " '",
                    text.get().bold().green(),
                    "' ",
                    "copied to clipboard",
                )));
            })
            .render(text!("Copy to clipboard")),
        Notifications::new().content_width(40).render()
    ]
}
