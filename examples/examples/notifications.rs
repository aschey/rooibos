use std::time::Duration;

use rooibos::components::{Notification, Notifications, Notifier};
use rooibos::reactive::{col, line, mount, wgt, Render};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{wasm_compat, Runtime};
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::widgets::{Block, Paragraph};
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
    wasm_compat::spawn_local(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        notifier.notify(Notification::new("notify 1"));
        tokio::time::sleep(Duration::from_secs(1)).await;
        notifier.notify(Notification::new("notify 2"));
    });
    col![
        wgt!(
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
