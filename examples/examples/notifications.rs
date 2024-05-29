use std::error::Error;
use std::io::Stdout;
use std::time::Duration;

use rooibos::components::{notifications, notify, Notification};
use rooibos::dom::{col, widget_ref, Render};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{Runtime, RuntimeSettings};
use rooibos::tui::text::Line;
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
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        notify(Notification::new("notify 1"));
        tokio::time::sleep(Duration::from_secs(1)).await;
        notify(Notification::new("notify 2"));
    });
    col![
        widget_ref!(
            Paragraph::new(vec![
                Line::from("text1"),
                Line::from("text2"),
                Line::from("text3"),
                Line::from("text4")
            ])
            .block(Block::bordered())
        ),
        notifications()
    ]
}
