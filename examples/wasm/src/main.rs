use std::error::Error;

use rooibos::runtime::backend::crossterm::CrosstermBackend;
use rooibos::runtime::Runtime;
use wasm_test::app;

#[rooibos::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::initialize(CrosstermBackend::stdout(), app);
    runtime.run().await?;
    Ok(())
}
