use std::error::Error;
use std::io::{IsTerminal, stdin};

use rooibos::reactive::{Render, line, mount, wgt};
use rooibos::runtime::Runtime;
use rooibos::terminal::crossterm::CrosstermBackend;
use rooibos::tui::style::Stylize;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[rooibos::main]
async fn main() -> Result<()> {
    let input = {
        let input = stdin();
        if input.is_terminal() {
            return Err("Pipe in some text. Ex: echo hi > cargo run --example=pipe_input")?;
        }

        let mut buffer = String::new();
        input.read_line(&mut buffer)?;
        buffer
    };

    mount(|| app(input));
    let runtime = Runtime::initialize(CrosstermBackend::stdout());
    runtime.run().await?;
    Ok(())
}

fn app(text: String) -> impl Render {
    wgt!(line!("You typed: ", text.clone().bold().green()))
}
