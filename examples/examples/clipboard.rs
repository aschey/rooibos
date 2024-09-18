use rooibos::components::{Button, Input, Notification, Notifications, Notifier};
use rooibos::reactive::graph::traits::Get;
use rooibos::reactive::layout::chars;
use rooibos::reactive::{col, mount, span, text, Render, UpdateLayoutProps};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{set_clipboard, Runtime};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::terminal::ClipboardKind;
use taffy::LengthPercentageAuto;

type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;

    Ok(())
}

fn app() -> impl Render {
    let notifier = Notifier::new();
    let textarea = Input::get_ref();

    let text = textarea.text();

    col![
        Input::default()
            .placeholder_text("Enter some text")
            .height(chars(1.))
            .margin_left(LengthPercentageAuto::Length(1.))
            .render(textarea),
        Button::new()
            .width(chars(25.))
            .height(chars(3.))
            .on_click(move || {
                set_clipboard(text.get(), ClipboardKind::Clipboard).unwrap();
                notifier.notify(Notification::new(span!(
                    " '{}' copied to clipboard",
                    text.get()
                )));
            })
            .render(text!("Copy to clipboard")),
        Notifications::new().width(40).render()
    ]
}
