use std::io::Stdout;

use rooibos::dom::{KeyCode, KeyEvent};
use rooibos::reactive::graph::effect::Effect;
use rooibos::reactive::graph::signal::signal;
use rooibos::reactive::graph::traits::{Get, Update};
use rooibos::reactive::{wgt, Render};
use rooibos::runtime::backend::crossterm::{CrosstermBackend, TerminalSettings};
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{set_title, Runtime};
type Result<T> = std::result::Result<T, RuntimeError>;

#[rooibos::main]
async fn main() -> Result<()> {
    let runtime = Runtime::initialize(
        CrosstermBackend::<Stdout>::new(TerminalSettings::default().title("initial title")),
        app,
    );
    runtime.run().await?;
    Ok(())
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let update_count = move || set_count.update(|c| *c += 1);

    Effect::new(move |prev: Option<()>| {
        let count = count.get();
        if prev.is_some() {
            set_title(format!("count {count}")).unwrap();
        }
    });

    let key_down = move |key_event: KeyEvent, _, _| {
        if key_event.code == KeyCode::Enter {
            update_count();
        }
    };

    wgt!(format!("count {}", count.get()))
        .on_key_down(key_down)
        .on_click(move |_, _, _| update_count())
}
