use std::error::Error;
use std::io::Stdout;

use rooibos::components::{notifications, Button, Notification, Notifier};
use rooibos::dom::{col, row, text, widget_ref, Constrainable, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::backend::ClipboardKind;
use rooibos::runtime::{set_clipboard, Runtime, RuntimeSettings};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::default(),
        app,
    );
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
            col![
                Button::new()
                    .on_click(move || {
                        set_clipboard(count.get(), ClipboardKind::Clipboard);
                        notifier.notify(Notification::new("Current count copied to clipboard"));
                    })
                    .render(text!("Copy to clipboard")),
            ]
            .length(21)
        ]
        .length(3),
        col![
            widget_ref!(format!("count: {}", count.get()))
                .on_key_down(key_down)
                .on_click(move |_, _| update_count()),
        ],
        notifications()
    ]
}
