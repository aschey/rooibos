use std::error::Error;
use std::io::Stdout;

use rooibos::dom::{widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::{execute, init_executor, run, start, RuntimeSettings, TerminalSettings};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    execute(async_main)
}

#[tokio::main()]
async fn async_main() -> Result<()> {
    init_executor(async {
        start(RuntimeSettings::default(), app);
        run::<Stdout>(TerminalSettings::default()).await?;
        Ok(())
    })
    .await
}

fn app() -> impl Render {
    let (count, set_count) = signal(0);

    let key_down = move |key_event: KeyEvent, _| {
        if key_event.code == KeyCode::Enter {
            set_count.update(|c| *c += 1);
        }
    };

    widget_ref!(format!("count {}", count.get()))
        .focusable(true)
        .on_key_down(key_down)
}
