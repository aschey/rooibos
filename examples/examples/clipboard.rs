use rooibos::components::{Button, Notification, Notifications, Notifier};
use rooibos::dom::layout::chars;
use rooibos::dom::{
    height, margin, row, text, wgt, width, KeyCode, KeyEvent, Render, UpdateLayoutProps,
};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::backend::ClipboardKind;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{set_clipboard, Runtime};
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
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

    row![
        props(height!(3.)),
        Button::new()
            .width(chars(21.))
            .on_click(move || {
                set_clipboard(count.get(), ClipboardKind::Clipboard).unwrap();
                notifier.notify(Notification::new(" Current count copied to clipboard "));
            })
            .render(text!("Copy to clipboard")),
        wgt!(
            props(width!(20.), margin!(1.)),
            format!("count: {}", count.get())
        )
        .on_key_down(key_down)
        .on_click(move |_, _| update_count()),
        Notifications::new().width(40).render()
    ]
}
