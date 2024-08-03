use std::error::Error;
use std::io::Stdout;
use std::time::Duration;

use rooibos::components::{Notification, Notifications, Notifier};
use rooibos::dom::{col, line, widget_ref, Render};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{wasm_compat, Runtime, RuntimeSettings};
use rooibos::tui::widgets::{Block, Paragraph};

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
    let notifier = Notifier::new();
    wasm_compat::spawn_local(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        notifier.notify(Notification::new("notify 1"));
        tokio::time::sleep(Duration::from_secs(1)).await;
        notifier.notify(Notification::new("notify 2"));
    });
    col![
        widget_ref!(
            Paragraph::new(vec![
                line!("text1"),
                line!("text2"),
                line!("text3"),
                line!("text4")
            ])
            .block(Block::bordered())
        ),
        Notifications::new().render()
    ]
}
