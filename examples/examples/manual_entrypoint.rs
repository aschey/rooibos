use rooibos::dom::{wgt, KeyCode, KeyEvent, Render};
use rooibos::reactive::signal::signal;
use rooibos::reactive::traits::{Get, Update};
use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::error::RuntimeError;
use rooibos::runtime::{execute, run_with_executor, Runtime};
type Result<T> = std::result::Result<T, RuntimeError>;

fn main() -> Result<()> {
    execute(async_main)
}

#[tokio::main]
async fn async_main() -> Result<()> {
    run_with_executor(async {
        let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
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

    wgt!(format!("count {}", count.get())).on_key_down(key_down)
}
