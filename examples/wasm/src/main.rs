use std::error::Error;

use rooibos::reactive::mount;
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
use wasm_test::app;

#[rooibos::main]
async fn main() -> Result<(), Box<dyn Error>> {
    mount(app);
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}
