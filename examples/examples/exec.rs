use std::env;
use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{exec, Runtime, RuntimeSettings};

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

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
        if key_event.code == KeyCode::Char('e') {
            let editor = env::var("EDITOR").unwrap_or("vim".to_string());
            exec(tokio::process::Command::new(editor), |_, _, _| {});
        }
    };

    widget_ref!(format!(
        "count {}. Press 'e' to open your editor.",
        count.get()
    ))
    .on_key_down(key_down)
    .on_click(move |_, _| update_count())
}
