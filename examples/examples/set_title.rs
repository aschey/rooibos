use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::effect::Effect;
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::runtime::{set_title, Runtime, RuntimeSettings};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize_with_settings(
        RuntimeSettings::default(),
        CrosstermBackend::<Stdout>::new(TerminalSettings::default().title("initial title")),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    Effect::new(move |prev| {
        let count = count.get();
        if prev.is_some() {
            set_title(format!("count {count}"));
        }
    });

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    widget_ref!(format!("count {}", count.get()))
        .on_key_down(key_down)
        .on_click(move |_, _| update_count())
}
