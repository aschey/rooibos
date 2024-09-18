use std::io::Stdout;
use std::time::Duration;

use rooibos::dom::{KeyCode, KeyEvent};
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{mount, wgt, Render};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{insert_before, Runtime};
use rooibos::terminal::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::tui::Viewport;
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::<Stdout>::new(
        TerminalSettings::default().viewport(Viewport::Inline(8)),
    ));
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            insert_before(1, "test").unwrap();
        }
    });

    wgt!(format!("count {}", count.get()))
        .on_key_down(key_down)
        .on_click(move |_, _, _| update_count())
}
