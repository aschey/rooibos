use std::error::Error;

use rooibos::components::{Button, Notification, Notifications, Notifier};
use rooibos::dom::{col, length, row, text, widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::backend::ClipboardKind;
use rooibos::runtime::{set_clipboard, Runtime, RuntimeSettings};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(RuntimeSettings::default(), CrosstermBackend::stdout(), app);
    runtime.run().await?;

    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);
    let update_count = move || set_count.update(|c| *c += 1);
    let notifier = Notifier::new();

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    col![
        row![
            props(length(3)),
            col![
                props(length(21)),
                Button::new()
                    .on_click(move || {
                        set_clipboard(count.get(), ClipboardKind::Clipboard);
                        notifier.notify(Notification::new(" Current count copied to clipboard "));
                    })
                    .render(text!("Copy to clipboard")),
            ]
        ],
        col![
            widget_ref!(format!("count: {}", count.get()))
                .on_key_down(key_down)
                .on_click(move |_, _| update_count()),
        ],
        Notifications::new().width(40).render()
    ]
}
