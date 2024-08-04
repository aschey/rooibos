use std::error::Error;

use rooibos::dom::{widget_ref, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::{execute, init_executor, Runtime, RuntimeSettings};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    execute(async_main)
}

#[tokio::main]
async fn async_main() -> Result<()> {
    init_executor(async {
        let runtime =
            Runtime::initialize(RuntimeSettings::default(), CrosstermBackend::stdout(), app);
        runtime.run().await?;
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

    widget_ref!(format!("count {}", count.get())).on_key_down(key_down)
}
