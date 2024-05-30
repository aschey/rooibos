use std::error::Error;
use std::io::Stdout;
use std::time::Duration;

use rooibos::dom::{widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::runtime::{insert_before, Runtime, RuntimeSettings};
use rooibos::tui::Viewport;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::new(TerminalSettings::default().viewport(Viewport::Inline(8))),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            insert_before(1, "test");
        }
    });

    widget_ref!(format!("count {}", count.get()))
        .on_key_down(key_down)
        .on_click(move |_, _| update_count())
}
